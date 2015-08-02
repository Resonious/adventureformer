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

pub struct CrattleCrute {
    position: Vec2<GLfloat>,
    frame:    GLint,
    flipped:  bool,
    angle:    GLfloat,

    // Vec2 colors are x=primary, y=secondary
    body_color:       Vec2<u32>,
    left_foot_color:  Vec2<u32>,
    right_foot_color: Vec2<u32>,
    eye_color: u32
}

impl CrattleCrute {
    fn sprite(&self, color_swap_1: Vec2<u32>, color_swap_2: Vec2<u32>) -> SpriteType2Color2 {
        SpriteType2Color2 {
            position: self.position,
            frame:    self.frame,
            flipped:  self.flipped as GLint,
            angle:    self.angle,
            color_swap_1: color_swap_1,
            color_swap_2: color_swap_2
        }
    }
    pub fn body_sprite(&self) -> SpriteType2Color2 {
        self.sprite(
            Vec2::new(0x0094FFFF, self.body_color.x),
            Vec2::new(0x00C7FFFF, self.body_color.y)
        )
    }
    pub fn left_foot_sprite(&self) -> SpriteType2Color2 {
        self.sprite(
            Vec2::new(0xFF0000FF, self.left_foot_color.x),
            Vec2::new(0xDB002FFF, self.left_foot_color.y)
        )
    }
    pub fn right_foot_sprite(&self) -> SpriteType2Color2 {
        self.sprite(
            Vec2::new(0xFF0000FF, self.right_foot_color.x),
            Vec2::new(0xDB002FFF, self.right_foot_color.y)
        )
    }
    pub fn eye_sprite(&self) -> SpriteType3Color1 {
        let offset = &TEST_OFFSETS[self.frame as usize];

        SpriteType3Color1 {
            position: self.position,
            frame:    0,
            flipped:  self.flipped as GLint,
            angle:    self.angle + offset.angle,
            focus:    Vec2::new(2, 0) - offset.pos,
            color_swap: Vec2::new(0x5900FFFF, self.eye_color)
        }
    }
}

pub struct GameData {
    pub fps: i32,
    pub time_counter: f32,
    pub frame_counter: i32,

    pub cam_pos: Vec2<GLfloat>,
    pub controls: Controls,
    pub player: CrattleCrute,
    pub player2: CrattleCrute
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

        let p1footcolor = Vec2::new(0xBB98E2FF, 0x9F67E0FF);
        game.player.body_color       = Vec2::new(0x0026FFFF, 0x1979FFFF);
        game.player.left_foot_color  = p1footcolor;
        game.player.right_foot_color = p1footcolor;
        game.player.eye_color        = 0xDD304AFF;

        let goddamn_color2 = Vec2::new(0xD66FC8FF, 0xD693E4FF);
        game.player2.body_color       = goddamn_color2;
        game.player2.left_foot_color  = goddamn_color2;
        game.player2.right_foot_color = goddamn_color2;
        game.player2.eye_color        = goddamn_color2.x;
        game.player2.position = Vec2::new(-40.0, -40.0);


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
        // TODO Just TWO players for now
        let plr_count = 2;
        gl_data.images.crattlecrute_front_foot.empty_buffer_data(plr_count, gl::DYNAMIC_DRAW);
        gl_data.images.crattlecrute_body.empty_buffer_data(plr_count, gl::DYNAMIC_DRAW);
        gl_data.images.crattlecrute_back_foot.empty_buffer_data(plr_count, gl::DYNAMIC_DRAW);
        gl_data.images.eye_1.empty_buffer_data(plr_count, gl::DYNAMIC_DRAW);
        gl_data.images.test_spin.empty_buffer_data(plr_count, gl::DYNAMIC_DRAW);
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
        game.player2.frame += 1;
        if game.player2.frame >= 9 { game.player2.frame = 1 }
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
            game.player.position.x -= 100.0 * delta_t;
            game.player.flipped = true;
        }
        if game.controls.right.down() {
            game.player.position.x += 100.0 * delta_t;
            game.player.flipped = false;
        }
        if game.controls.up.down() {
            if game.player.flipped {
                game.player.angle -= 3.14159 * delta_t;
            }
            else {
                game.player.angle += 3.14159 * delta_t;
            }
        }
        if game.controls.down.down() {
            if game.player.flipped {
                game.player.angle += 3.14159 * delta_t;
            }
            else {
                game.player.angle -= 3.14159 * delta_t;
            }
        }
        if game.controls.debug.just_down() {
            // println!("Time: {}", time);

            // game.player.angle = 0.0;
            game.player.frame += 1;
            if game.player.frame >= 9 { game.player.frame = 1 }
        }
        if delta_t < 0.0 {
            println!("Delta time < 0!!! {}", delta_t);
        }

        macro_rules! plrdata {
            ($($img:ident|$render:ident|$sprite:ty),+) => {
                $({
                    gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.$img.vbo);
                    let buffer  = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
                    let sprites = slice::from_raw_parts_mut::<$sprite>(
                        transmute(buffer),
                        // NOTE
                        // This is contingent on how large the buffer we made in the
                        // loading routine is.
                        2 // TODO <- number of players
                    );
                    // This should be a loop through sorted draw calls on for this texture.
                    sprites[0] = game.player.$render();
                    sprites[1] = game.player2.$render();
                    gl::UnmapBuffer(gl::ARRAY_BUFFER);
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                });*
            }
        };
        plrdata!(
            eye_1                   | eye_sprite        | SpriteType3Color1,
            crattlecrute_front_foot | right_foot_sprite | SpriteType2Color2,
            crattlecrute_body       | body_sprite       | SpriteType2Color2,
            crattlecrute_back_foot  | left_foot_sprite  | SpriteType2Color2
        );

        // === Draw test spinning body ===
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_data.images.test_spin.vbo);
        let buffer = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
        let sprites = slice::from_raw_parts_mut::<SpriteType3Color1>(
            transmute(buffer),
            1
        );

        sprites[0] = SpriteType3Color1 {
            position: Vec2::new(game.player.angle, game.player.position.y),
            frame:    0,
            flipped:  !game.player.flipped as GLint,
            angle:    game.player.angle,
            focus:    Vec2::new(21, 52),
            color_swap: Vec2::new(0x0094FFFF, game.player.eye_color)
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
            ($img:expr, $count:expr) => {
                $img.set();
                gl::DrawElementsInstanced(
                // TODO last argument here will be number of render calls counted for player!
                    gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), $count
                );
            }
        };

        renderthing!(gl_data.images.test_spin, 1);
        renderthing!(gl_data.images.crattlecrute_back_foot, 2);
        renderthing!(gl_data.images.crattlecrute_body, 2);
        renderthing!(gl_data.images.crattlecrute_front_foot, 2);
        renderthing!(gl_data.images.eye_1, 2);
    }

    window.swap_buffers();
}

#[test]
fn it_works() {
}
