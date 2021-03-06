#[macro_export]
macro_rules! image_assets {
    ($($texcoords_name:ident $name:ident: $sprite_type:ident [$texcoords:expr][$w:expr;$h:expr] $path:expr),+) =>  {

    pub struct Images {
        $(
        pub $name: ImageAsset,
        pub $texcoords_name: [Texcoords; $texcoords]
        // concat_idents!($name, _texcoords): [Texcoords, $num_texcoords]
        ),*
    }

    impl Images {
        pub fn init(&mut self, gl_data: *const GLData) {
            $(
            self.$name = ImageAsset {
                gl_data: gl_data,
                filename: $path,
                vbo: 0,
                set_attributes: $sprite_type::set,
                shader: $sprite_type::shader,
                attributes_size: size_of::<$sprite_type>(),
                texture: unsafe { zeroed() },
                frame_width: $w,
                frame_height: $h,
                texcoord_count: $texcoords
            };
            )*
        }
    }
    }
}

macro_rules! vertex_attrib_pointer {
    (float, $loc:expr, $size:expr, $offset:expr) => {
        gl::VertexAttribPointer(
            $loc, 1, gl::FLOAT, gl::FALSE as GLboolean,
            $size, transmute($offset)
        );
    };
    (vec2, $loc:expr, $size:expr, $offset:expr) => {
        gl::VertexAttribPointer(
            $loc, 2, gl::FLOAT, gl::FALSE as GLboolean,
            $size, transmute($offset)
        );
    };
    (ivec2, $loc:expr, $size:expr, $offset:expr) => {
        gl::VertexAttribIPointer(
            $loc, 2, gl::INT,
            $size, transmute($offset)
        );
    };
    ($int_type:ident, $loc:expr, $size:expr, $offset:expr) => {
        gl::VertexAttribIPointer(
            $loc, 1, gl::INT,
            $size, transmute($offset)
        );
    };
}

#[macro_export]
macro_rules! shader_assets {
    (
        $(
        $sprite_type:ident:
        [vertex]
            $(layout (location = $loc:expr) in $glsltype:ident($attrtype:ty) $name:ident;)*
            ($vertmain:expr)
        [fragment]
            ($fragmain:expr)
        )+
    ) => {
        pub struct Shader {
            pub program:             GLuint,
            pub cam_pos_uniform:     GLint,
            pub scale_uniform:       GLint,
            pub sprite_size_uniform: GLint,
            pub screen_size_uniform: GLint,
            pub tex_uniform:         GLint,
            pub frames_uniform:      GLint,
        }

        #[allow(non_snake_case)]
        pub struct Shaders {
            $( pub $sprite_type: Shader ),*
        }

        impl Shaders {
            pub fn count() -> usize {
                let mut c = 0;
                $(
                    $fragmain; // compiler complains if we don't reference stuff in loop.
                    c += 1;
                )*
                c
            }

            pub fn compile(gl_data: &mut GLData, window: &glfw::Window) -> Vec<&'static str> {
                let mut failed = Vec::<&'static str>::with_capacity(Shaders::count());
                let cam_pos_str     = CString::new("cam_pos".to_string()).unwrap();
                let scale_str       = CString::new("scale".to_string()).unwrap();
                let sprite_size_str = CString::new("sprite_size".to_string()).unwrap();
                let screen_size_str = CString::new("screen_size".to_string()).unwrap();
                let tex_str         = CString::new("tex".to_string()).unwrap();
                let frames_str      = CString::new("frames".to_string()).unwrap();

                $(unsafe {
                    println!("About to compile {}", stringify!($sprite_type));
                    let ref mut shader = gl_data.shaders.$sprite_type;

                    let existing_program =
                        if gl::IsProgram(shader.program) == gl::TRUE {
                            Some(shader.program)
                        } else { None };

                    shader.program = match render::create_program($sprite_type::vertex_shader(), $sprite_type::fragment_shader()) {
                        Some(program) => {
                            gl::UseProgram(program);

                            shader.cam_pos_uniform = gl::GetUniformLocation(program, cam_pos_str.as_ptr());
                            shader.scale_uniform = gl::GetUniformLocation(program, scale_str.as_ptr());

                            shader.sprite_size_uniform = gl::GetUniformLocation(program, sprite_size_str.as_ptr());
                            shader.screen_size_uniform = gl::GetUniformLocation(program, screen_size_str.as_ptr());
                            shader.tex_uniform = gl::GetUniformLocation(program, tex_str.as_ptr());
                            shader.frames_uniform = gl::GetUniformLocation(program, frames_str.as_ptr());

                            // TODO this should maybe be handled elsewhere
                            gl::Uniform1f(shader.scale_uniform, 2.0);
                            match window.get_size() {
                                (width, height) => gl::Uniform2f(shader.screen_size_uniform, width as f32, height as f32)
                            }

                            match existing_program {
                                Some(existing) => gl::DeleteProgram(existing),
                                _ => {}
                            }

                            program
                        }

                        None => {
                            failed.push(stringify!($sprite_type));
                            shader.program
                        }
                    }
                })*

                failed
            }

            pub fn each_shader<F>(&mut self, mut f: F) where F: FnMut(&mut Shader, &'static str) {
                $( f(&mut self.$sprite_type, stringify!($sprite_type)); )*
            }
        }

        $(
            #[derive(Clone)]
            pub struct $sprite_type {
                $( pub $name: $attrtype ),*
            }
            impl Copy for $sprite_type { }

            impl $sprite_type {
                #[allow(unused_assignments)] // Compiler is wrong about offset not being used...
                pub fn set(vbo: GLuint) { unsafe {
                    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

                    let size_of_sprite = size_of::<$sprite_type>() as GLint;
                    let mut offset: i64 = 0;

                    $(
                        gl::EnableVertexAttribArray($loc);
                        vertex_attrib_pointer!(
                            $glsltype, $loc,
                            size_of_sprite, offset
                        );
                        gl::VertexAttribDivisor($loc, 1);
                        offset += size_of::<$attrtype>() as i64;
                    )*

                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                }}

                pub fn shader<'a>(gl_data: &'a GLData) -> &'a Shader {
                    &gl_data.shaders.$sprite_type
                }

                pub fn vertex_shader() -> String {
                    let mut vertex = String::with_capacity(4092);
                    vertex.push_str("
                        #version 300 es
                        precision mediump float;

                        // Per vertex, normalized:
                        layout (location = 0) in vec2 vertex_pos;
                        // Per instance:
                    ");
                    $({
                        if $loc == 0 {
                            panic!("Shader location 0 is reserved for vertex position")
                        }
                        vertex.push_str(&format!("layout (location = {}) in {} {};\n",
                            $loc, stringify!($glsltype), stringify!($name)
                        ));
                    });*

                    vertex.push_str("
                        // NOTE up this if you run into problems
                        uniform vec2[256] frames;
                        uniform vec2 screen_size;
                        uniform vec2 cam_pos;     // in pixels
                        uniform vec2 sprite_size; // in pixels
                        uniform float scale;

                        out vec2 texcoord;

                        const vec2 TEXCOORD_FROM_ID[4] = vec2[4](
                            vec2(1.0, 1.0), vec2(1.0, 0.0),
                            vec2(0.0, 0.0), vec2(0.0, 1.0)
                        );

                        vec2 from_pixel(vec2 pos)
                        {
                            return pos / screen_size;
                        }
                        ivec2 from_pixel(ivec2 pos)
                        {
                            return pos / ivec2(screen_size);
                        }

                        int flipped_vertex_id()
                        {
                            return 3 - gl_VertexID;
                        }

                        // These are I suppose an optimization for squares
                        // rotating about their centers.
                        const float VERT_DIST = 1.41421356237;
                        const float ANGLE_OFFSETS[4] = float[4](
                            // pi/4
                            0.78539816339,
                            // 7pi/4
                            5.49778714378,
                            // 5pi/4
                            3.92699081699,
                            // 3pi/4
                            2.35619449019
                        );

                        vec4 color_from(int color)
                        {
                            // Totally assumes little endian...
                            int red = int((uint(color) & 0xFF000000u) >> 24u);
                            return vec4(
                                float(red) / 256.0,
                                float((color & 0x00FF0000) >> 16) / 256.0,
                                float((color & 0x0000FF00) >> 8)  / 256.0,
                                float( color & 0x000000FF)        / 256.0
                            );
                        }
                    ");
                    vertex.push_str($vertmain);
                    // println!("VERTEX:\n{}", vertex);
                    vertex
                }

                pub fn fragment_shader() -> String {
                    let mut fragment = String::with_capacity(1028);
                    fragment.push_str("
                        #version 300 es
                        precision mediump float;

                        in vec2 texcoord;
                        out vec4 color;
                        uniform sampler2D tex;

                        bool approx(vec4 a, vec4 b, float alpha)
                        {
                            vec4 diff = abs(a - b);
                            return diff.x <= alpha &&
                                   diff.y <= alpha &&
                                   diff.z <= alpha &&
                                   diff.w <= alpha;
                        }
                    ");
                    fragment.push_str($fragmain);

                    // println!("FRAGMENT:\n{}", fragment);
                    fragment
                }
            }
        )*
    }
}
