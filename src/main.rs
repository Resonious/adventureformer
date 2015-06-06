#![feature(libc)]

extern crate libc;

// use std::mem::transmute;
use std::ptr;
use libc::{c_int, c_void, c_char};
use std::ffi::CString;

struct GLFWwindow;
#[link(name = "glfw3dll")]
#[allow(improper_ctypes)]
extern {
    fn glfwInit() -> bool;
    fn glfwCreateWindow(
        w: c_int,
        h: c_int,
        title: *const c_char,
        monitor: *const c_void,
        share: *const c_void) -> *mut GLFWwindow;
    fn glfwTerminate();
    fn glfwMakeContextCurrent(window: *mut GLFWwindow);
    fn glfwWindowShouldClose(window: *mut GLFWwindow) -> bool;
    fn glfwSwapBuffers(window: *mut GLFWwindow);
    fn glfwPollEvents();
}

fn main() { unsafe {
    println!("Hello, world!");
    if !glfwInit() { panic!("Couldn't initialize GLFW"); }

    let window_title = CString::new(&b"Holy shit"[..]).unwrap();
    let window = glfwCreateWindow(640, 480, window_title.as_ptr(), ptr::null(), ptr::null());
    if window as isize == 0 { glfwTerminate(); panic!("Couldn't create window"); }

    glfwMakeContextCurrent(window);

    while !glfwWindowShouldClose(window) {
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    glfwTerminate();
}}
