extern crate glfw;
extern crate gl;
extern crate libc;

pub mod vecmath;
pub mod render;
pub mod assets;
pub mod controls;

use gl::types::*;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};
use libc::{c_void};
use vecmath::Vec2;
use std::mem::{size_of, size_of_val, transmute};
use std::ptr;
use std::slice;
use render::{GLData};
use assets::{SpriteType2Color2, SpriteType3Color1};
use controls::Controls;
use std::f32::consts::PI;

macro_rules! check_error(
    () => (
        match gl::GetError() {
            gl::NO_ERROR => {}
            gl::INVALID_ENUM => panic!("Invalid enum!"),
            gl::INVALID_VALUE => panic!("Invalid value!"),
            gl::INVALID_OPERATION => panic!("Invalid operation!"),
            gl::INVALID_FRAMEBUFFER_OPERATION => panic!("Invalid framebuffer operation?!"),
            gl::OUT_OF_MEMORY => panic!("Out of memory bro!!!!!!!"),
            e => panic!("OpenGL Error: {:?}", e)
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

pub struct GameData {
    pub fps: i32,
    pub time_counter: f32,
    pub frame_counter: i32,

    pub cam_pos: Vec2<GLfloat>,

    pub controls: Controls,

    pub player_pos: Vec2<GLfloat>,
    pub player_angle: GLfloat,
    pub flip_player: bool,
    pub player_frame: GLint
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
        // Defying borrow checker here:
        let gl_data_ptr: usize = transmute(gl_data as *const GLData);
        gl_data.images.init(transmute(gl_data_ptr));

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
        let failed = assets::Shaders::compile(gl_data, window);
        if failed.len() > 0 {
            panic!("Failed to compile: {:?}", failed);
        }
        gl_data.shaders.each_shader(|shader, _name| {
            gl::Uniform2f(shader.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);
        });

        gl_data.images.crattlecrute_front_foot.load();
        gl_data.images.crattlecrute_body.load();
        gl_data.images.crattlecrute_back_foot.load();
        gl_data.images.eye_1.load();
        gl_data.images.test_spin.load();
        // TODO Just one player for now
        gl_data.images.crattlecrute_front_foot.empty_buffer_data(1, gl::DYNAMIC_DRAW);
        gl_data.images.crattlecrute_body.empty_buffer_data(1, gl::DYNAMIC_DRAW);
        gl_data.images.crattlecrute_back_foot.empty_buffer_data(1, gl::DYNAMIC_DRAW);
        gl_data.images.eye_1.empty_buffer_data(1, gl::DYNAMIC_DRAW);
        gl_data.images.test_spin.empty_buffer_data(1, gl::DYNAMIC_DRAW);
    }
    else {
        let failed = assets::Shaders::compile(gl_data, window);
        if failed.len() > 0 {
            println!("Shaders: {:?} failed to compile and were not reloaded.", failed);
        }
        gl_data.shaders.each_shader(|shader, _name| {
            gl::Uniform2f(shader.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);
        });
    }
}

struct Offset {
    pub pos: Vec2<GLint>,
    pub angle: GLfloat
}

// THIS IS GENERATED
static TEST_OFFSETS: [Offset; 9] = [
    Offset { pos: Vec2::<GLint> { x: 16, y: 21 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 17, y: 21 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 18, y: 20 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 18, y: 20 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 17, y: 20 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 17, y: 21 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 18, y: 20 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 18, y: 20 }, angle: 0.000000 },
    Offset { pos: Vec2::<GLint> { x: 17, y: 20 }, angle: 0.000000 },
];

#[no_mangle]
pub extern "C" fn update(
    game:    &mut GameData,
    gl_data: &mut GLData,
    delta_t: f32,
    glfw:    &mut glfw::Glfw,
    window:  &mut glfw::Window,
    events:  &Receiver<(f64, glfw::WindowEvent)>,
    time:    i64
) {
    // === Count Frames Per Second ===
    game.time_counter += delta_t;
    game.frame_counter += 1;

    if game.time_counter >= 1.0 {
        game.fps = game.frame_counter;
        game.frame_counter = 0;
        game.time_counter = 0.0;
    }

    glfw.poll_events();

    // === INPUT ===
    let mut new_window_size: Option<(GLfloat, GLfloat)> = None;

    for control in game.controls.iter_mut() {
        control.last_frame = control.this_frame;
    }
    for (_, event) in glfw::flush_messages(&events) {
        match event {
            // TODO maybe keep this around but this is actually stupid
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            glfw::WindowEvent::Key(key, _, action, _) => {
                let control = match key {
                    Key::W  => &mut game.controls.up,
                    Key::Up => &mut game.controls.up,

                    Key::S    => &mut game.controls.down,
                    Key::Down => &mut game.controls.down,

                    Key::A    => &mut game.controls.left,
                    Key::Left => &mut game.controls.left,

                    Key::D     => &mut game.controls.right,
                    Key::Right => &mut game.controls.right,

                    Key::B => &mut game.controls.debug,

                    _ => break
                };

                match action {
                    Action::Press => control.this_frame = true,
                    Action::Release => control.this_frame = false,
                    _ => {}
                }
            }

            glfw::WindowEvent::Size(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
                new_window_size = Some((width as GLfloat, height as GLfloat));
            },

            _ => {}
        }
    }

    // === PROCESSING? ===
    unsafe {
        if game.controls.left.down() {
            game.player_pos.x -= 100.0 * delta_t;
            game.flip_player = true;
        }
        if game.controls.right.down() {
            game.player_pos.x += 100.0 * delta_t;
            game.flip_player = false;
        }
        if game.controls.up.down() {
            if game.flip_player {
                game.player_angle -= 3.14159 * delta_t;
            }
            else {
                game.player_angle += 3.14159 * delta_t;
            }
        }
        if game.controls.down.down() {
            if game.flip_player {
                game.player_angle += 3.14159 * delta_t;
            }
            else {
                game.player_angle -= 3.14159 * delta_t;
            }
        }
        if game.controls.debug.just_down() {
            // println!("Time: {}", time);

            // game.player_angle = 0.0;
            game.player_frame += 1;
            if game.player_frame >= 9 { game.player_frame = 1 }
        }
        if delta_t < 0.0 {
            println!("Delta time < 0!!! {}", delta_t);
        }

        macro_rules! plrdata {
            ($($img:ident),+) => {
                $({
                gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.$img.vbo);
                let buffer  = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
                let sprites = slice::from_raw_parts_mut::<SpriteType2Color2>(
                    transmute(buffer),
                    1 // TODO <- number of players
                );
                // This should be a loop through sorted draw calls on for this texture.
                sprites[0] = SpriteType2Color2 {
                    position: game.player_pos,
                    frame:    game.player_frame,
                    flipped:  game.flip_player as GLint,
                    angle:    game.player_angle,
                    color_swap_1: Vec2::new(0x0094FFFF, 0x2B06D3FF),
                    color_swap_2: Vec2::new(0x00C7FFFF, 0x3071D3FF)
                };
                gl::UnmapBuffer(gl::ARRAY_BUFFER);
                });*
            }
        };
        plrdata!(crattlecrute_front_foot, crattlecrute_body, crattlecrute_back_foot);

        // === Draw test eye ===
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.eye_1.vbo);
        let buffer = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
        let sprites = slice::from_raw_parts_mut::<SpriteType3Color1>(
            transmute(buffer),
            1
        );

        let eye_color = (((game.player_angle / PI * 310.0) as u32) << 8) | 0x000000FF;
        let eye_offset = &TEST_OFFSETS[game.player_frame as usize];

        sprites[0] = SpriteType3Color1 {
            position: game.player_pos,
            frame:    0,
            flipped:  game.flip_player as GLint,
            angle:    game.player_angle + eye_offset.angle,
            focus:    Vec2::new(2, 0) - eye_offset.pos,
            color_swap: Vec2::new(0x5900FFFF, eye_color)
        };
        gl::UnmapBuffer(gl::ARRAY_BUFFER);

        // === Draw test spinning body ===
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.test_spin.vbo);
        let buffer = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
        let sprites = slice::from_raw_parts_mut::<SpriteType3Color1>(
            transmute(buffer),
            1
        );

        sprites[0] = SpriteType3Color1 {
            position: Vec2::new(game.player_angle, game.player_pos.y),
            frame:    0,
            flipped:  game.flip_player as GLint,
            angle:    game.player_angle,
            focus:    Vec2::new(21, 52),
            color_swap: Vec2::new(0x0094FFFF, eye_color)
        };
        gl::UnmapBuffer(gl::ARRAY_BUFFER);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // === RENDER ===
    unsafe {
        gl_data.shaders.each_shader(|shader, _name| {
            gl::UseProgram(shader.program);
            match new_window_size {
                Some((width, height)) =>
                    gl::Uniform2f(shader.screen_size_uniform, width, height),
                None => {}
            }
            gl::Uniform2f(shader.cam_pos_uniform, game.cam_pos.x, game.cam_pos.y);
        });

        gl::ClearColor(0.2, 0.2, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        macro_rules! renderthing {
            ($img:expr) => {
                $img.set();
                gl::DrawElementsInstanced(
                // TODO last argument here will be number of render calls counted for player!
                    gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), 1
                );
            }
        };

        renderthing!(gl_data.images.test_spin);
        renderthing!(gl_data.images.crattlecrute_back_foot);
        renderthing!(gl_data.images.crattlecrute_body);
        renderthing!(gl_data.images.crattlecrute_front_foot);
        renderthing!(gl_data.images.eye_1);
    }

    window.swap_buffers();
}

#[test]
fn it_works() {
}
