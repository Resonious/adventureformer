extern crate glfw;
extern crate gl;
extern crate libc;

pub mod vecmath;
pub mod render;

use gl::types::*;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};
use libc::{c_void};
use vecmath::Vec2;
use std::ffi::CString;
use std::mem::{size_of, size_of_val, transmute};
use std::ptr;
use render::{Texture, Frame, Texcoords, SpriteData};

static SQUARE_VERTICES: [GLfloat; 8] = [
//    position
     2.0,  2.0, //   1.0, 1.0, // Top right
     2.0,  0.0, //   1.0, 0.0, // Bottom right
     0.0,  0.0, //   0.0, 0.0, // Bottom left
     0.0,  2.0, //   0.0, 1.0  // Top left
];
static SQUARE_INDICES: [GLuint; 6] = [
    0, 1, 3,
    1, 2, 3
];

macro_rules! stride(
    ($val:expr) => (($val * size_of::<GLfloat>() as i32))
);

pub struct GLData {
    // === Global buffers ===
    pub vao: GLuint,
    pub square_vbo: GLuint,
    pub square_ebo: GLuint,

    // === Shader information ===
    pub shader_prog:         GLuint,
    pub cam_pos_uniform:     GLint,
    pub scale_uniform:       GLint,
    pub sprite_size_uniform: GLint,
    pub screen_size_uniform: GLint,
    pub tex_uniform:         GLint,
    pub frames_uniform:      GLint,

    // === Hardcoded textures/texcoords ===
    pub front_foot_tex:       Texture,
    pub front_foot_texcoords: [Texcoords; 9],
    pub body_tex:       Texture,
    pub body_texcoords: [Texcoords; 9],
    pub back_foot_tex:       Texture,
    pub back_foot_texcoords: [Texcoords; 9],
}

pub struct GameData {
    cam_pos: Vec2<f32>,

    front_foot_frames: [Frame; 9],
    body_frames:       [Frame; 9],
    back_foot_frames:  [Frame; 9]

        // TODO place a SpriteData somewhere and see how it goes
}

extern "C" {
    // Supplied by Resonious' glfw fork
    fn glfwSet(new_glfw: *const c_void);
}

#[no_mangle]
pub unsafe extern "C" fn load(
    first_load: bool,
    game:       &mut GameData,
    gl_data:    &mut GLData,
    glfw:       &glfw::Glfw,
    window:     &mut glfw::Window,
    glfw_data:  *const c_void,
) {
    println!("LOAD!");
    glfwSet(glfw_data);
    gl::load_with(|s| window.get_proc_address(s));
    if first_load {
        game.cam_pos.x = 0.0;
        game.cam_pos.y = 0.0;

        // === Crattlecrute textures ===
        gl_data.front_foot_tex = render::load_texture("assets/crattlecrute/front-foot.png");
        gl_data.front_foot_tex.add_frames(&mut game.front_foot_frames, 90, 90);

        gl_data.body_tex = render::load_texture("assets/crattlecrute/body.png");
        gl_data.body_tex.add_frames(&mut game.body_frames, 90, 90);

        gl_data.back_foot_tex = render::load_texture("assets/crattlecrute/back-foot.png");
        gl_data.back_foot_tex.add_frames(&mut game.back_foot_frames, 90, 90);

        // === Blending for alpha ===
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // === Global VAO ===
        gl::GenVertexArrays(1, &mut gl_data.vao);
        gl::BindVertexArray(gl_data.vao);

        // === Basic sprite square vertex buffer ===
        gl::GenBuffers(1, &mut gl_data.square_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.square_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&SQUARE_VERTICES) as GLsizeiptr,
            transmute(&SQUARE_VERTICES[0]),
            gl::STATIC_DRAW
        );
        gl::EnableVertexAttribArray(render::ATTR_VERTEX_POS);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean,
                                stride!(2), ptr::null());

        // === Basic sprite square element buffer ===
        gl::GenBuffers(1, &mut gl_data.square_ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gl_data.square_ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            size_of_val(&SQUARE_INDICES) as GLsizeiptr,
            transmute(&SQUARE_INDICES[0]),
            gl::STATIC_DRAW
        );

        if !compile_shaders(gl_data, game, window) {
            panic!("Cannot continue due to shader error.");
        }
        gl_data.front_foot_tex.generate_texcoords_buffer(&mut gl_data.front_foot_texcoords);
        gl_data.body_tex.generate_texcoords_buffer(&mut gl_data.body_texcoords);
        gl_data.back_foot_tex.generate_texcoords_buffer(&mut gl_data.back_foot_texcoords);
    }
    else {
        if !compile_shaders(gl_data, game, window) {
            println!("Shaders not reloaded.");
        }
    }
}

fn compile_shaders(gl_data: &mut GLData, game: &GameData, window: &glfw::Window) -> bool {
    let existing_program = unsafe {
        if gl::IsProgram(gl_data.shader_prog) == gl::TRUE {
            Some(gl_data.shader_prog)
        } else { None }
    };

    gl_data.shader_prog = match render::create_program(render::STANDARD_VERTEX, render::STANDARD_FRAGMENT) {
        Some(program) => program,
        None          => return false
    };
    unsafe {
        gl::UseProgram(gl_data.shader_prog);

        let cam_pos_str         = CString::new("cam_pos".to_string()).unwrap();
        gl_data.cam_pos_uniform = gl::GetUniformLocation(gl_data.shader_prog, cam_pos_str.as_ptr());

        let scale_str         = CString::new("scale".to_string()).unwrap();
        gl_data.scale_uniform = gl::GetUniformLocation(gl_data.shader_prog, scale_str.as_ptr());

        let sprite_size_str         = CString::new("sprite_size".to_string()).unwrap();
        gl_data.sprite_size_uniform = gl::GetUniformLocation(gl_data.shader_prog, sprite_size_str.as_ptr());

        let screen_size_str         = CString::new("screen_size".to_string()).unwrap();
        gl_data.screen_size_uniform = gl::GetUniformLocation(gl_data.shader_prog, screen_size_str.as_ptr());

        let tex_str         = CString::new("tex".to_string()).unwrap();
        gl_data.tex_uniform = gl::GetUniformLocation(gl_data.shader_prog, tex_str.as_ptr());

        let frames_str         = CString::new("frames".to_string()).unwrap();
        gl_data.frames_uniform = gl::GetUniformLocation(gl_data.shader_prog, frames_str.as_ptr());

        gl::Uniform2f(gl_data.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);
        gl::Uniform1f(gl_data.scale_uniform, 2.0);
        match window.get_size() {
            (width, height) => gl::Uniform2f(gl_data.screen_size_uniform, width as f32, height as f32)
        }

        match existing_program {
            Some(program) => gl::DeleteProgram(program),
            _ => {}
        }
    }

    true
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
