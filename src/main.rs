#![feature(std_misc, negate_unsigned)]

extern crate gl;
extern crate glfw;
extern crate libc;

#[cfg(windows)]
pub mod win32;
#[cfg(windows)]
use win32 as platform;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(target_os = "macos")]
pub mod osx;
#[cfg(target_os = "macos")]
use osx as platform;

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
            move || platform::watch_for_updated_game_lib(&game_lib_sender)
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
    let ticks_per_second = platform::query_performance_frequency() as f32;

    while !window.should_close() {
        unsafe {
            platform::query_performance_counter(&mut this_frame_time);

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
