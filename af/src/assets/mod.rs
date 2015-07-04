extern crate gl;
extern crate glfw;

use gl::types::*;
use std::mem::{zeroed, size_of, transmute};
use render;
use render::{GLData, ImageAsset, Texcoords};
use std::ffi::CString;
use vecmath::*;

#[macro_use]
mod macros;

image_assets!(
    ccbdy crattlecrute_body:       SpriteType1 [9][90;90] "assets/crattlecrute/body.png",
    ccbft crattlecrute_back_foot:  SpriteType1 [9][90;90] "assets/crattlecrute/back-foot.png",
    ccfft crattlecrute_front_foot: SpriteType1 [9][90;90] "assets/crattlecrute/front-foot.png"
);

shader_assets!(
SpriteType1:

    [vertex]
        layout (location = 1) in vec2(Vec2<GLfloat>) position; // in pixels
        layout (location = 2) in int(GLint) frame;
        layout (location = 3) in int(GLint) flipped;   // actually a bool
    ("
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
     }
     ")

    [fragment]
    ("
     void main()
     {
        color = texture(tex, texcoord);
     }
     ")

SpriteType1Color2:

    [vertex]
        layout (location = 1) in vec2(Vec2<GLfloat>) position; // in pixels
        layout (location = 2) in int(GLint) frame;
        layout (location = 3) in int(GLint) flipped;   // actually a bool
        layout (location = 4) in ivec2(Vec2<GLuint>) color_swap_1;
        layout (location = 5) in ivec2(Vec2<GLuint>) color_swap_2;
    ("
     out vec4 cswap1_from;
     out vec4 cswap1_to;
     out vec4 cswap2_from;
     out vec4 cswap2_to;

     vec4 color_from(int color)
     {
         return vec4(
             float((color & 0xFF000000) >> 24) / 256,
             float((color & 0x00FF0000) >> 16) / 256,
             float((color & 0x0000FF00) >>  8) / 256,
             float(color & 0x000000FF)         / 256
         );
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

         cswap1_from = color_from(color_swap_1.x);
         cswap1_to   = color_from(color_swap_1.y);
         cswap2_from = color_from(color_swap_2.x);
         cswap2_to   = color_from(color_swap_2.y);
     }
     ")

    [fragment]
    ("
     in vec4 cswap1_from;
     in vec4 cswap1_to;
     in vec4 cswap2_from;
     in vec4 cswap2_to;

     void main()
     {
        color = texture(tex, texcoord);

        if (color == cswap1_from)
            color = cswap1_to;
        else if (color == cswap2_from)
            color = cswap2_to;
     }
     ")
);

