extern crate glfw;
extern crate gl;
extern crate libc;

pub mod vecmath;
pub mod render;
pub mod assets;

use gl::types::*;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};
use libc::{c_void};
use vecmath::Vec2;
use std::ffi::CString;
use std::mem::{size_of, size_of_val, transmute};
use std::ptr;
use std::slice;
use render::{SpriteType1};

macro_rules! check_error(
    () => (
        match gl::GetError() {
            gl::NO_ERROR => {}
            gl::INVALID_ENUM => panic!("Invalid enum!"),
            gl::INVALID_VALUE => panic!("Invalid value!"),
            gl::INVALID_OPERATION => panic!("Invalid operation!"),
            gl::INVALID_FRAMEBUFFER_OPERATION => panic!("Invalid framebuffer operation?!"),
            gl::OUT_OF_MEMORY => panic!("Out of memory bro!!!!!!!"),
            // gl::STACK_UNDERFLOW => panic!("Stack UNDERflow!"),
            // gl::STACK_OVERFLOW => panic!("Stack overflow!")
            _ => panic!("I DON'T KNOW. FULL BANANNACAKES.")
        }
    )
);

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
    /*
    pub front_foot_tex:       Texture,
    pub front_foot_texcoords: [Texcoords; 9],
    pub body_tex:       Texture,
    pub body_texcoords: [Texcoords; 9],
    pub body_vbo:       GLuint,
    pub back_foot_tex:       Texture,
    pub back_foot_texcoords: [Texcoords; 9],
    */

    pub images: assets::Images
}

#[test]
fn how_big_is_image_asset_data() {
    let size_of_image_asset = (size_of::<Texture>() + size_of::<Texcoords>()*10 + size_of::<GLuint>()) as f64;
    let number_of_images = 500.0;
    let total_in_kilobytes = size_of_image_asset * number_of_images / 1000.0;
    panic!("{} image asseta = {} kb", number_of_images, total_in_kilobytes);
}

pub struct GameData {
    pub cam_pos: Vec2<f32>,

    // pub front_foot_frames: [Frame; 9],
    // pub body_frames:       [Frame; 9],
    // pub back_foot_frames:  [Frame; 9],
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
        // ============== Game ================
        game.cam_pos.x = 0.0;
        game.cam_pos.y = 0.0;

        // ============== OpenGL ================
        gl_data.images.init();

        // === Crattlecrute textures ===
        // gl_data.front_foot_tex = render::load_texture("assets/crattlecrute/front-foot.png");
        // gl_data.front_foot_tex.add_frames(&mut game.front_foot_frames, 90, 90);

        // gl_data.body_tex = render::load_texture("assets/crattlecrute/body.png");
        // gl_data.body_tex.add_frames(&mut game.body_frames, 90, 90);

        // gl_data.back_foot_tex = render::load_texture("assets/crattlecrute/back-foot.png");
        // gl_data.back_foot_tex.add_frames(&mut game.back_foot_frames, 90, 90);

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

        // === Shaders and texcoords ===
        if !compile_shaders(gl_data, game, window) {
            panic!("Cannot continue due to shader error.");
        }
        // gl_data.front_foot_tex.generate_texcoords_buffer(&mut gl_data.front_foot_texcoords);
        // gl_data.back_foot_tex.generate_texcoords_buffer(&mut gl_data.back_foot_texcoords);
        // gl_data.body_tex.generate_texcoords_buffer(&mut gl_data.body_texcoords);

        gl_data.images.crattlecrute_body.load();
        let mut body_vbo = &mut gl_data.images.crattlecrute_body.vbo;
        gl::GenBuffers(1, body_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, *body_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            // TODO Just one player for now
            1 * size_of::<SpriteType1>() as GLsizeiptr,
            ptr::null(),
            gl::DYNAMIC_DRAW
        );
        // Also only body for now (fuckin' get those all into one goddamn image)
    }
    else {
        if !compile_shaders(gl_data, game, window) {
            println!("Due to errors, shaders were not reloaded.");
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
        gl::Uniform1f(gl_data.scale_uniform, 1.0);
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

    // === INPUT ===
    let mut flip_player = false;
    for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                println!("YOU PRESSED {:?}", key);
                if key == Key::Space { flip_player = true }
            }

            glfw::WindowEvent::Size(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
                gl::Uniform2f(gl_data.screen_size_uniform, width as f32, height as f32);
            },

            _ => {}
        }
    }
     
    // === PROCESSING? ===
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.crattlecrute_body.vbo);
        let body_buffer = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
        let body_sprites = slice::from_raw_parts_mut::<SpriteType1>(
            transmute(body_buffer),
            1 // TODO <- number of players
        );
        // This should be a loop through sorted draw calls on for this texture.
        body_sprites[0] = SpriteType1 {
            position: Vec2::new(10.0, 10.0),
            frame: 0,
            flipped: flip_player as GLint
        };
        gl::UnmapBuffer(gl::ARRAY_BUFFER);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // === RENDER ===
    unsafe {
        gl::Uniform2f(gl_data.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);

        gl::ClearColor(0.1, 0.1, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let ref crattlecrute_body = gl_data.images.crattlecrute_body;
        crattlecrute_body.texture.set(
            gl_data.tex_uniform,
            gl_data.sprite_size_uniform,
            gl_data.frames_uniform,
            90.0, 90.0
        );
        SpriteType1::set(crattlecrute_body.vbo);
        gl::DrawElementsInstanced(
            // TODO last argument here will be number of render calls counted for player!
            gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), 1
        );

        check_error!();
    }

    window.swap_buffers();
}

#[test]
fn it_works() {
}
