extern crate glfw;
extern crate gl;

use gl::types::*;
use std::sync::mpsc::Receiver;

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
    glfw_data:  *const u8,
) {
    println!("LOAD yes");
    if first_load { println!("FIRST!"); }
    else          { println!("CONSECUTIVE!!"); }
}

#[no_mangle]
pub extern "C" fn update(
    game:    &mut GameData,
    gl_data: &mut GLData,
    glfw:    &glfw::Glfw,
    window:  &glfw::Window,
    event:   &Receiver<(f64, glfw::WindowEvent)>
) {
    println!("UPDATE? RENDER?");
}

#[test]
fn it_works() {
}
