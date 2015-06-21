#![feature(std_misc, negate_unsigned, fs_time)]

extern crate gl;
extern crate glfw;
extern crate libc;

use std::path::Path;
use std::fs;
use std::fs::File;
use glfw::{Action, Context, Key};
use libc::{c_char, c_void, c_int};
use std::dynamic_lib::DynamicLibrary;
use std::thread;
// use std::c_str::ToCStr;
use std::mem::{transmute, uninitialized, drop};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::ffi::CString;

type LoadFn = extern "C" fn (
    bool, // first load?
    &mut u8, // GameData
    &mut u8, // GLData
    &glfw::Glfw,
    &glfw::Window,
    *const u8 // glfw _data
);

type UpdateFn = extern "C" fn (
    &mut u8, // GameData
    &mut u8, // GLData
    &glfw::Glfw,
    &glfw::Window,
    &Receiver<(f64, glfw::WindowEvent)>
);

// Windows shit
#[repr(C)]
struct Win32SecurityAttributes {
    length: i32, // always size_off::<Win32SecurityAttributes>()
    security_descriptor: *const c_void,
    inherit_handle:      bool
}

#[repr(C)]
struct Win32FileNotifyInformation {
    next_entry_offset: i32,
    action:            i32,
    file_name_length:  i32,
    first_file_name_char: c_char
}

extern "C" {
    pub fn CreateFile(
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
    ) -> bool;

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

    pub fn GetLastError() -> c_int;
}
const INFINITE: i32 = 0xFFFFFFFF;
const FILE_NOTIFY_CHANGE_LAST_WRITE: i32 = 0x00000010;
const INVALID_HANDLE_VALUE: *const c_void = -1 as *const c_void;

const FILE_LIST_DIRECTORY: i32 = 1;

const FILE_SHARE_DELETE: i32 = 0x00000004;
const FILE_SHARE_READ:   i32 = 0x00000001;
const FILE_SHARE_WRITE:  i32 = 0x00000002;

const FILE_ACTION_ADDED:    i32 = 0x00000001;
const FILE_ACTION_MODIFIED: i32 = 0x00000003;

// Glfw shit
extern "C" {
    static _glfw: u8;
}

fn copy_game_lib_to_cwd() {
    match fs::copy("./af/target/debug/af.dll", "./af.dll") {
        Err(e) => panic!("Couldn't copy af.dll: {}", e),
        _ => {}
    }
}

fn load_game_lib() -> DynamicLibrary {
    let dylib_path = Path::new("./af.dll");

    match DynamicLibrary::open(Some(dylib_path)) {
        Ok(lib) => lib,
        Err(e)  => panic!("Couldn't load game lib: {}", e)
    }
}

fn load_symbols_from(lib: &DynamicLibrary) -> (LoadFn, UpdateFn) {
    unsafe {
        let load: LoadFn = match lib.symbol::<u8>("update") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Couldn't grab update symbol from game lib! {}", e)
        };

        let update: UpdateFn = match lib.symbol::<u8>("load") {
            Ok(f) => transmute(f),
            Err(e) => panic!("Couldn't grab load symbol from game lib! {}", e)
        };

        (load, update)
    }
}

unsafe fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    let dylib_dir  = Path::new("./af/target/debug/");
    let dylib_path = Path::new("./af/target/debug/af.dll");

    let mut last_modified = match File::open(dylib_path) {
        Ok(file) => match file.metadata() {
            Ok(m)  => m.modified(),
            Err(e) => panic!("Couldn't fetch dll metadata: {}", e)
        },
        Err(e) => panic!("Couldn't open {}: {}", dylib_path.display(), e)
    };

    let dylib_dir_str = CString::new(dylib_dir.to_str().unwrap()).unwrap();
    let handle = FindFirstChangeNotificationA(dylib_dir_str.as_ptr(), false, FILE_NOTIFY_CHANGE_LAST_WRITE);

    if handle == INVALID_HANDLE_VALUE {
        panic!("Failed to acquire file change notification handle from Windows: {}", GetLastError());
    }

    loop {
        match WaitForSingleObject(handle, INFINITE) {
            // File was changed or created
            0x00000000 => {
                let lib_file = match File::open(dylib_path) {
                    Ok(f)  => f,
                    Err(e) => continue
                };
                let new_last_modified = match lib_file.metadata() {
                    Ok(m)  => m.modified(),
                    Err(e) => continue
                };

                if new_last_modified != last_modified {
                    println!("UPDATE THE DYLIB NOWW!!!");
                    last_modified = new_last_modified;
                    sender.send(());
                }
            }

            0xFFFFFFFF => panic!("Error occurred during directory wait! {}", GetLastError()),

            _ => println!("Something happened but don't care.")
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

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
        thread::spawn(move || watch_for_updated_game_lib(&game_lib_sender));
        // load(true, &_glfw, &window, &mut game_memory[0], &mut options_memory[0], &mut gl_memory[0]);
    }
    while !window.should_close() {
        match game_lib_receiver.try_recv() {
            Ok(()) => {
                drop(game_lib);
                copy_game_lib_to_cwd();
                game_lib = load_game_lib();
                match load_symbols_from(&game_lib) { (l, u) => { load = l; update = u } }
            }
            _ => {}
        }

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
    }
}
