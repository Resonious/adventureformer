extern crate gl;
extern crate libc;

use vecmath::Vec2;
use gl::types::*;
use std::ffi::CString;
use libc::{c_char, c_int, c_void};
use std::mem::{uninitialized, transmute, size_of, size_of_val};
use std::ptr;
use std::slice;
use std::vec::Vec;

extern "C" {
    fn stbi_load(
        filename: *const c_char,
        x: *mut c_int,
        y: *mut c_int,
        components: *mut c_int,
        force_components: c_int
    ) -> *const u8;

    fn stbi_image_free(ptr: *const u8);
}

// NOTE make sure these constants match what's in the shader.
pub static ATTR_VERTEX_POS: u32 = 0;
pub static ATTR_POSITION: u32 = 1;
pub static ATTR_FRAME: u32 = 2;
pub static ATTR_FLIPPED: u32 = 3;

pub static FRAME_UNIFORM_MAX: i64 = 256;

pub static STANDARD_VERTEX: &'static str = "
        #version 330 core

        // Per vertex, normalized:
        layout (location = 0) in vec2 vertex_pos;
        // Per instance:
        layout (location = 1) in vec2 position;       // in pixels
        layout (location = 2) in int frame;
        layout (location = 3) in int flipped; // actually a bool

        uniform vec2[256] frames;
        uniform vec2 screen_size;
        uniform vec2 cam_pos;     // in pixels
        uniform vec2 sprite_size; // in pixels
        uniform float scale;

        out vec2 texcoord;
        int call_index = 0;

        const vec2 TEXCOORD_FROM_ID[4] = vec2[4](
            vec2(1.0, 1.0), vec2(1.0, 0.0),
            vec2(0.0, 0.0), vec2(0.0, 1.0)
        );

        vec2 from_pixel(vec2 pos)
        {
            return pos / screen_size;
        }

        int flipped_vertex_id()
        {
            return 3 - gl_VertexID;
        }

        void main()
        {
            vec2 pixel_screen_pos = (position - cam_pos) * 2;
            gl_Position = vec4(
                (vertex_pos * from_pixel(sprite_size) + from_pixel(pixel_screen_pos)) * scale,
                0.0f, 1.0f
            );
            int index = flipped != 0 ? flipped_vertex_id() : gl_VertexID;
            if (frame == -1)
                texcoord = TEXCOORD_FROM_ID[index];
            else
                texcoord = frames[frame * 4 + index];
            texcoord.y = 1 - texcoord.y;
            call_index += 1;
            if (call_index >= 6) call_index = 0;
        }
    ";

pub static STANDARD_FRAGMENT: &'static str = "
        #version 330 core
        in vec2 texcoord;
        out vec4 color;
        uniform sampler2D tex;
        void main()
        {
            color = texture(tex, texcoord);
        }
    ";

macro_rules! check_log(
    ($typ:expr, $get_iv:ident | $get_log:ident $val:ident $status:ident $on_error:ident) => (
        unsafe {
            let mut status = 0;
            gl::$get_iv($val, gl::$status, &mut status);
            if status == 0 {
                let mut len = 0;
                gl::$get_iv($val, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize - 1);
                gl::$get_log($val, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                
                $on_error!("{} ERROR: {:?}", $typ, String::from_utf8(buf));
                false
            } else {
                println!("I THINK THE {} COMPILED", $typ);
                true
            }
        }
    )
);

macro_rules! make_shader(
    ($name:ident: $shader_type:ident) => (
        unsafe {
            let sh = gl::CreateShader(gl::$shader_type);
            let shader_src_str = CString::new($name.to_string()).unwrap();
            gl::ShaderSource(sh, 1, &shader_src_str.as_ptr(), ptr::null());
            gl::CompileShader(sh);
            sh
        }
    )
);

// TODO in the future, we can do SpriteType2 that adds rotation/scaling etc.
#[derive(Clone)]
pub struct SpriteType1 {
    pub position: Vec2<GLfloat>,
    pub frame: GLint,
    pub flipped: GLint
}
impl Copy for SpriteType1 { }

impl SpriteType1 {
    pub fn set(vbo: GLuint) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let size_of_sprite = size_of::<SpriteType1>() as GLint;
            assert_eq!(size_of_sprite, 16);

            // == Position ==
            gl::EnableVertexAttribArray(ATTR_POSITION);
            gl::VertexAttribPointer(
                ATTR_POSITION, 2, gl::FLOAT, gl::FALSE as GLboolean,
                size_of_sprite, transmute(0 as isize)
            );
            gl::VertexAttribDivisor(ATTR_POSITION, 1);
            let mut offset = 2 * size_of::<GLfloat>() as i64;
            assert_eq!(offset, 8);

            // == Frame ==
            gl::EnableVertexAttribArray(ATTR_FRAME);
            gl::VertexAttribIPointer(
                ATTR_FRAME, 1, gl::INT,
                size_of_sprite, transmute(offset)
            );
            gl::VertexAttribDivisor(ATTR_FRAME, 1);
            offset += 1 * size_of::<GLint>() as i64;
            assert_eq!(offset, 12);

            // == Flipped ==
            gl::EnableVertexAttribArray(ATTR_FLIPPED);
            gl::VertexAttribIPointer(
                ATTR_FLIPPED, 1, gl::INT,
                size_of_sprite, transmute(offset)
            );
            gl::VertexAttribDivisor(ATTR_FLIPPED, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

pub struct Texcoords {
    pub top_right:    Vec2<GLfloat>,
    pub bottom_right: Vec2<GLfloat>,
    pub bottom_left:  Vec2<GLfloat>,
    pub top_left:     Vec2<GLfloat>
}

impl Texcoords {
    pub unsafe fn copy_to(&self, dest: *mut Texcoords) {
        ptr::copy(self, dest, 1);
    }
}

// Represents an animation frame; a square section of a Texture.
pub struct Frame {
    pub position: Vec2<f32>,
    pub size: Vec2<f32>,

    // Texcoords are generated via #generate_texcoords.
    pub texcoords: Texcoords
}

impl Frame {
    pub fn generate_texcoords(&mut self, tex_width: f32, tex_height: f32) {
        let ref position  = self.position;
        let ref size      = self.size;

        // TODO SIMD this son of a bitch
        self.texcoords = Texcoords {
            top_right: Vec2::new(
                (position.x + size.x) / tex_width,
                (position.y + size.y) / tex_height
            ),

            bottom_right: Vec2::new(
                (position.x + size.x) / tex_width,
                (position.y)          / tex_height
            ),

            bottom_left: Vec2::new(
                (position.x)          / tex_width,
                (position.y)          / tex_height
            ),

            top_left: Vec2::new(
                (position.x)          / tex_width,
                (position.y + size.y) / tex_height
            )
        };
    }
}

// Represents an actual texture that is currently on the GPU.
#[allow(missing_copy_implementations)]
pub struct Texture {
    pub id: GLuint,
    pub width: i32,
    pub height: i32,
    pub filename: &'static str,
    pub frame_texcoords_size: i64,
    pub texcoords_space: *mut [Texcoords]
}

impl Texture {
    pub fn set_full(&self, sampler_uniform: GLint, sprite_size_uniform: GLint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::Uniform1i(sampler_uniform, 0);
            gl::Uniform2f(sprite_size_uniform, self.width as f32, self.height as f32);
        }
    }

    #[inline]
    pub fn texcoords(&self) -> &[Texcoords] {
        unsafe { transmute(self.texcoords_space) }
    }

    #[inline]
    pub fn texcoords_mut(&mut self) -> &mut [Texcoords] {
        unsafe { transmute(self.texcoords_space) }
    }

    // NOTE this expects #generate_texcoords_buffer to have been called
    // if there are frames. "
    pub fn set(&self, sampler_uniform:     GLint,
                      sprite_size_uniform: GLint,
                      frames_uniform:      GLint,
                      width: f32, height: f32) {
        unsafe {
            assert!(self.frame_texcoords_size / 8 < FRAME_UNIFORM_MAX);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::Uniform1i(sampler_uniform, 0);
            gl::Uniform2f(sprite_size_uniform, width as f32, height as f32);

            let frames_len = self.texcoords().len();

            if frames_len > 0 {
                gl::Uniform2fv(
                    frames_uniform,
                    frames_len as GLint * 4,
                    transmute(&(&*self.texcoords_space)[0])
                );
            }
        }
    }

    fn put_texcoord(&mut self, index: usize, texcoord: Texcoords) {
        self.texcoords_mut()[index] = texcoord;
    }

    // NOTE this should be properly merged with add_frames.
    pub fn generate_texcoords_buffer(
        &mut self, frame_width: usize, frame_height: usize, space: *mut [Texcoords]
    ) {
        unsafe {
            let frames_len = (*space).len();

            let mut frames = Vec::<Frame>::with_capacity(frames_len);
            self.add_frames(&mut frames, frame_width, frame_height);
            assert_eq!(frames.len(), frames_len); // PLZ

            self.texcoords_space = space;
            for i in (0..frames_len) {
                frames[i].texcoords.copy_to(&mut self.texcoords_mut()[i]);
            }
        }
    }

    // Fill the given slice with frames of the given width and height. "
    // So this is now called only by #generate_texcoords_buffer
    pub fn add_frames(&mut self, space: &mut Vec<Frame>, uwidth: usize, uheight: usize) {
        let count = space.capacity();
        let tex_width  = self.width as f32;
        let tex_height = self.height as f32;
        let width  = uwidth as f32;
        let height = uheight as f32;

        {
            let mut current_pos = Vec2::<f32>::new(0.0, tex_height - height);

            for i in (0..count) {
                if current_pos.x + width > tex_width {
                    current_pos.x = 0.0;
                    current_pos.y -= height;
                }
                if current_pos.y < 0.0 {
                    panic!(
                        "Too many frames! Asked for {} {}x{} frames on a {}x{} texture.",
                        count, width, height, tex_width, tex_height
                    );
                }

                let mut frame = Frame {
                    position:  current_pos,
                    size:      Vec2::new(width, height),
                    texcoords: unsafe { uninitialized() }
                };
                frame.generate_texcoords(tex_width, tex_height);
                space.push(frame);

                current_pos.x += width;
            }
        }

        self.frame_texcoords_size = size_of::<Texcoords>() as i64 * count as i64;
    }

    // TODO man, should this be a destructor?
    // A: NO
    pub fn unload(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

// NOTE don't instantiate these willy nilly!
pub struct ImageAsset {
    pub filename:       &'static str,
    pub vbo:            GLuint,
    pub texture:        Texture,
    pub frame_width:    usize,
    pub frame_height:   usize,
    pub texcoord_count: usize,
    // The next texcoord_count * size_of::<Texcoords>() bytes
    // should be free for this struct to use.
}

impl ImageAsset {
    pub unsafe fn texcoords(&mut self) -> &mut [Texcoords] {
        let count_ptr: *mut usize = &mut self.texcoord_count;
        slice::from_raw_parts_mut::<Texcoords>(
            transmute(count_ptr.offset(1)),
            self.texcoord_count
        )
    }

    pub fn loaded(&self) -> bool { self.vbo != 0 }

    pub unsafe fn load(&mut self) {
        let mut texture = load_texture(self.filename);
        texture.generate_texcoords_buffer(self.frame_width, self.frame_height, self.texcoords());
        self.texture = texture;

        gl::GenBuffers(1, &mut self.vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        // NOTE that this doesn't do gl::BufferData for now.
    }

    pub unsafe fn unload(&mut self) {
        panic!("Unloading doesn't work yet hahahaha!");
    }
}

// Load a texture from the given filename into the GPU
// memory, returning a struct holding the OpenGL ID and
// dimensions.
pub fn load_texture(filename: &'static str) -> Texture {
    let mut width = 0; let mut height = 0; let mut comp = 0;
    let mut tex_id: GLuint = 0;

    unsafe {
        let cfilename = CString::new(filename.to_string()).unwrap();
        let img = stbi_load(cfilename.as_ptr(), &mut width, &mut height, &mut comp, 4);
        assert_eq!(comp, 4);

        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        println!("Sending {} to GPU. Width: {} Height: {}", filename, width, height);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32,
            width, height, 0, gl::RGBA,
            gl::UNSIGNED_BYTE, transmute(img)
        );

        stbi_image_free(img);
    }

    Texture {
        id: tex_id,
        width: width,
        height: height,
        filename: filename,
        frame_texcoords_size: 0,
        texcoords_space: &mut []
    }
}

pub fn create_program(vert: &str, frag: &str) -> Option<GLuint> {
    let vert_id = make_shader!(vert: VERTEX_SHADER);
    let vert_result: bool = check_log!(
        "VERTEX SHADER",
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
        println
    );
    if !vert_result {
        unsafe { gl::DeleteShader(vert_id); }
        return None;
    }

    let frag_id = make_shader!(frag: FRAGMENT_SHADER);
    let frag_result: bool = check_log!(
        "FRAGMENT SHADER",
        GetShaderiv | GetShaderInfoLog
        vert_id COMPILE_STATUS
        println
    );
    if !frag_result {
        unsafe { gl::DeleteShader(vert_id); }
        unsafe { gl::DeleteShader(frag_id); }
        return None;
    }

    let program_id = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program_id, vert_id);
        gl::AttachShader(program_id, frag_id);
        gl::LinkProgram(program_id);
    }

    let link_result = check_log!(
        "SHADER PROGRAM",
        GetProgramiv | GetProgramInfoLog
        program_id LINK_STATUS
        println
    );
    if !link_result {
        unsafe { gl::DeleteProgram(program_id); }
        unsafe { gl::DeleteShader(vert_id); }
        unsafe { gl::DeleteShader(frag_id); }
        return None;
    }

    unsafe {
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
    }

    Some(program_id)
}

