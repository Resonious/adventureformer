extern crate glfw;
extern crate gl;

use gl::types::*;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};

pub struct GLData {
    dummy: i64
}

pub struct GameData {
    dummy: i64
}

extern "C" {
    static _glfw: u8;
}

#[no_mangle]
pub extern "C" fn load(
    first_load: bool,
    game:       &mut GameData,
    gl_data:    &mut GLData,
    glfw:       &glfw::Glfw,
    window:     &glfw::Window,
    // glfw_data:  *const u8,
) {
    println!("LOAD!");
    if first_load { println!("FIRST!"); }
    else          { println!("CONSECUTIVE!!"); }
}

#[no_mangle]
pub extern "C" fn update(
    game:    &mut GameData,
    gl_data: &mut GLData,
    glfw:    &mut glfw::Glfw,
    window:  &mut glfw::Window,
    events:  &Receiver<(f64, glfw::WindowEvent)>
) {
    glfw.poll_events();

    for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                println!("YOU PRESSED {:?}", key);
            }
            _ => {}
        }
    }
}

#[test]
fn it_works() {
}
