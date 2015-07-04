#![feature(std_misc, negate_unsigned)]

extern crate gl;
extern crate glfw;
extern crate libc;

use std::path::Path;
use std::fs;
use glfw::{Context};
use libc::{c_char, c_void, c_int};
use std::dynamic_lib::DynamicLibrary;
use std::thread;
// use std::c_str::ToCStr;
use std::mem::{transmute, uninitialized, drop};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::ffi::CString;
use std::ptr;
use std::slice;

type LoadFn = extern "C" fn (
    bool, // first load?
    &mut u8, // GameData
    &mut u8, // GLData
    &glfw::Glfw,
    &glfw::Window,
    *const c_void // glfw _data
);

type UpdateFn = extern "C" fn (
    &mut u8, // GameData
    &mut u8, // GLData
    f32, // delta time
    &glfw::Glfw,
    &glfw::Window,
    &Receiver<(f64, glfw::WindowEvent)>
);

// Windows shit
#[repr(C)]
#[cfg(windows)]
pub struct Win32SecurityAttributes {
    length: i32, // always size_off::<Win32SecurityAttributes>()
    security_descriptor: *const c_void,
    inherit_handle:      bool
}

#[cfg(windows)]
#[repr(C)]
pub struct Win32FileNotifyInformation {
    next_entry_offset: i32,
    action:            i32,
    file_name_length:  i32,
    first_file_name_char: u16
}

#[cfg(windows)]
impl Win32FileNotifyInformation {
    pub fn file_name(&self) -> String { unsafe {
        let v = slice::from_raw_parts(&self.first_file_name_char, self.file_name_length as usize);
        String::from_utf16_lossy(v)
    }}
}

#[cfg(windows)]
extern "C" {
    pub fn CreateFileA(
        file_name:            *const c_char,
        desired_access:       i32,
        share_mode:           i32,
        security_attributes:  *const Win32SecurityAttributes,
        creation_disposition: i32,
        flags_and_attributes: i32,
        template_file:        *const c_void
    ) -> *const c_void;

    pub fn ReadDirectoryChangesW(
        directory:          *const c_void, // Retrieved from CreateFile
        buffer:             *const c_void, // Gets dynamically filled with Win32FileNotifyInformation
        buffer_length:      i32,
        watch_subtree:      bool,
        notify_filter:      i32,
        bytes_returned:     *const i32,
        overlapped:         *const c_void,
        completion_routine: *const c_void
    ) -> i32;

    pub fn FindFirstChangeNotificationA(
        path:          *const c_char,
        watch_subtree: bool,
        filter:        c_int
    ) -> *const c_void;

    pub fn FindNextChangeNotification(handle: *const c_void) -> bool;

    pub fn WaitForSingleObject(
        handle:     *const c_void,
        timeout_ms: c_int
    ) -> c_int;

    pub fn QueryPerformanceCounter(out: *mut i64) -> bool;
    pub fn QueryPerformanceFrequency(out: *mut i64) -> bool;

    pub fn GetLastError() -> c_int;
}
const FILE_NOTIFY_CHANGE_LAST_WRITE: i32 = 0x00000010;
const INVALID_HANDLE_VALUE: *const c_void = -1 as *const c_void;

const FILE_LIST_DIRECTORY: i32 = 1;

const FILE_SHARE_DELETE: i32 = 0x00000004;
const FILE_SHARE_READ:   i32 = 0x00000001;
const FILE_SHARE_WRITE:  i32 = 0x00000002;

const FILE_FLAG_BACKUP_SEMANTICS: i32 = 0x02000000;

const OPEN_EXISTING: i32 = 3;


static GAME_LIB_DIR: &'static str = "./af/target/debug/";
#[cfg(unix)]
static GAME_LIB_PATH: &'static str = "./af/target/debug/libaf.so";
#[cfg(windows)]
static GAME_LIB_PATH: &'static str = "./af/target/debug/af.dll";

#[cfg(unix)]
static GAME_LIB_FILE: &'static str = "./libaf.so";
#[cfg(windows)]
static GAME_LIB_FILE: &'static str = "./af.dll";

// Glfw shit
extern "C" {
    pub static _glfw: *const c_void;
}

fn copy_game_lib_to_cwd() {
    match fs::copy(GAME_LIB_PATH, GAME_LIB_FILE) {
        Err(e) => panic!("Couldn't copy {}: {}", GAME_LIB_PATH, e),
        _ => {}
    }
}

fn load_game_lib() -> DynamicLibrary {
    let dylib_path = Path::new(GAME_LIB_FILE);

    match DynamicLibrary::open(Some(dylib_path)) {
        Ok(lib) => lib,
        Err(e)  => panic!("Couldn't load game lib: {}", e)
    }
}

fn load_symbols_from(lib: &DynamicLibrary) -> (LoadFn, UpdateFn) {
    unsafe {
        let load: LoadFn = match lib.symbol::<u8>("load") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Couldn't grab update symbol from game lib! {}", e)
        };

        let update: UpdateFn = match lib.symbol::<u8>("update") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Couldn't grab load symbol from game lib! {}", e)
        };

        (load, update)
    }
}

#[cfg(windows)]
fn query_performance_frequency() -> i64 {
    let mut freq = 0i64;
    unsafe {
        if !QueryPerformanceFrequency(&mut freq) {
            panic!("Couldn't query performance frequency. Error code {}.", GetLastError());
        }
    }
    freq
}
#[cfg(windows)]
fn query_performance_counter(counter: &mut i64) {
    unsafe {
        if !QueryPerformanceCounter(counter) {
            panic!("Couldn't query performance counter. Error code {}.", GetLastError());
        }
    }
}

#[cfg(windows)]
unsafe fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    let dylib_dir  = Path::new(GAME_LIB_DIR);

    let dylib_dir_str = CString::new(dylib_dir.to_str().unwrap()).unwrap();
    let handle = CreateFileA(
        dylib_dir_str.as_ptr(),
        FILE_LIST_DIRECTORY,
        FILE_SHARE_DELETE|FILE_SHARE_READ|FILE_SHARE_WRITE,
        ptr::null(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        ptr::null()
    );
    if handle == INVALID_HANDLE_VALUE {
        match GetLastError() {
            5 => panic!("CreateFile for {} failed: Access denied", dylib_dir.display()),
            error_code => panic!("CreateFile for {} failed: Error code {}", dylib_dir.display(), error_code)
        }
    }

    let results_buffer = [0u8; 1024];
    let results_size: i32 = 0;

    loop {
        match ReadDirectoryChangesW(
            handle,
            transmute(&results_buffer[0]),
            results_buffer.len() as i32,
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE,
            &results_size,
            ptr::null(),
            ptr::null()
        ) {
            0 => println!("Failed to listen for a lib change! {}", GetLastError()),

            _ => {
                let result = transmute::<_, &Win32FileNotifyInformation>(&results_buffer[0]);
                if result.next_entry_offset != 0 {
                    panic!("YO, there are multiple entries. Handle that shit.");
                }
                let file_name = result.file_name();

                // NOTE Windows seems to just give back garbage string sizes, so
                // this file name is 'af.dll' fused with 'af.metadata.o'
                if file_name == "af.dlladata." {
                    sender.send(()).unwrap();
                }
            }
        }
    }
}

#[cfg(unix)]
fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    println!("on linux machine - no hot code update for now!");
}

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw
        .create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    gl::load_with(|s| window.get_proc_address(s));

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    let mut game_memory = unsafe { Box::new([uninitialized::<u8>(); 4096]) };
    let mut gl_memory   = unsafe { Box::new([uninitialized::<u8>(); 1024]) };


    copy_game_lib_to_cwd();
    let mut game_lib = load_game_lib();
    let (mut load, mut update) = load_symbols_from(&game_lib);

    let (game_lib_sender, game_lib_receiver) = channel();
    unsafe {
        let _t = thread::Builder::new().name("Game Lib Updater".to_string()).spawn(
            move || watch_for_updated_game_lib(&game_lib_sender)
        );
        load(
            true,
            transmute(&mut game_memory[0]),
            transmute(&mut gl_memory[0]),
            &glfw, &window,
            _glfw
        );
    }

    let mut last_frame_time = 0i64;
    let mut this_frame_time = 0i64;
    let ticks_per_second = query_performance_frequency() as f32;

    while !window.should_close() {
        unsafe {
            query_performance_counter(&mut this_frame_time);

            match game_lib_receiver.try_recv() {
                Ok(()) => {
                    drop(game_lib);
                    copy_game_lib_to_cwd();
                    game_lib = load_game_lib();
                    match load_symbols_from(&game_lib) { (l, u) => { load = l; update = u } }

                    load(
                        false,
                        transmute(&mut game_memory[0]),
                        transmute(&mut gl_memory[0]),
                        &glfw, &window,
                        _glfw
                    );
                }
                _ => {}
            }

            let delta_time =
                if last_frame_time == 0 { 1.0/60.0 }
                else { ((this_frame_time - last_frame_time) as f32) / ticks_per_second };

            update(
                transmute(&mut game_memory[0]),
                transmute(&mut gl_memory[0]),
                delta_time,
                &glfw, &window, &events
            );

            last_frame_time = this_frame_time;
        }
    }
}
