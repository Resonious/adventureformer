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
    ccbdy crattlecrute_body:       SpriteType2Color2 [9][90;90] "assets/crattlecrute/body.png",
    ccbft crattlecrute_back_foot:  SpriteType2Color2 [9][90;90] "assets/crattlecrute/back-foot.png",
    ccfft crattlecrute_front_foot: SpriteType2Color2 [9][90;90] "assets/crattlecrute/front-foot.png",
    ceye1 eye_1:     SpriteType3Color1 [1][4;5] "assets/eyes/standard-eye.png",
    tstsp test_spin: SpriteType3Color1 [9][90;90] "assets/crattlecrute/body.png"
);

shader_assets!(
// No rotation or color swapping - just frames and flipping.
SpriteType1:

    [vertex]
        layout (location = 1) in vec2(Vec2<GLfloat>) position; // in pixels
        layout (location = 2) in int(GLint) frame;
        layout (location = 3) in int(GLint) flipped;   // actually a bool
    ("
     void main()
     {
        vec2 pixel_screen_pos = (position - cam_pos) * 2.0;
        gl_Position = vec4(
            (vertex_pos * from_pixel(sprite_size) + from_pixel(pixel_screen_pos)) * scale,
            0.0f, 1.0f
        );
        int index = flipped != 0 ? flipped_vertex_id() : gl_VertexID;
        if (frame == -1)
            texcoord = TEXCOORD_FROM_ID[index];
        else
            texcoord = frames[frame * 4 + index];
        texcoord.y = 1.0 - texcoord.y;
     }
     ")

    [fragment]
    ("
     void main()
     {
        color = texture(tex, texcoord);
     }
     ")

// Frames, flipping, rotates around center, 2 colors can be swapped.
SpriteType2Color2:

    [vertex]
        layout (location = 1) in vec2(Vec2<GLfloat>) position; // in pixels
        layout (location = 2) in int(GLint) frame;
        layout (location = 3) in int(GLint) flipped;   // actually a bool
        layout (location = 4) in float(GLfloat) angle;
        layout (location = 5) in ivec2(Vec2<GLuint>) color_swap_1;
        layout (location = 6) in ivec2(Vec2<GLuint>) color_swap_2;
    ("
     out vec4 cswap1_from;
     out vec4 cswap1_to;
     out vec4 cswap2_from;
     out vec4 cswap2_to;

     void main()
     {
         vec2 pixel_screen_pos = (position - cam_pos) * 2.0;

         float vert_angle = angle + ANGLE_OFFSETS[gl_VertexID];
         vec2 vert = VERT_DIST * vec2(cos(vert_angle), sin(vert_angle)) + vec2(1.0, 1.0);

         gl_Position = vec4(
             (vert * from_pixel(sprite_size) + from_pixel(pixel_screen_pos)) * scale,
             0.0f, 1.0f
         );
         int index = flipped != 0 ? flipped_vertex_id() : gl_VertexID;
         if (frame == -1)
             texcoord = TEXCOORD_FROM_ID[index];
         else
             texcoord = frames[frame * 4 + index];
         texcoord.y = 1.0 - texcoord.y;

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

        if (approx(color, cswap1_from, 0.1))
            color = cswap1_to;
        else if (approx(color, cswap2_from, 0.1))
            color = cswap2_to;
     }
     ")

// Rotates around given focal point, 1 color swap.
SpriteType3Color1:

    [vertex]
        layout (location = 1) in vec2(Vec2<GLfloat>) position; // in pixels
        layout (location = 2) in int(GLint) frame;
        layout (location = 3) in int(GLint) flipped;   // actually a bool
        layout (location = 4) in float(GLfloat) angle;
        layout (location = 5) in ivec2(Vec2<GLint>) focus; // in pixels
        layout (location = 6) in ivec2(Vec2<GLuint>) color_swap;
    ("
     out vec4 cswap_from;
     out vec4 cswap_to;

     void main()
     {
         vec2 pixel_screen_pos = (position - cam_pos) * 2.0;
         vec2 rel_focus = vec2(focus) / sprite_size * 2.0;

         vec2 vert_offset = vertex_pos - rel_focus;
         float vert_angle = angle + atan(vert_offset.y, vert_offset.x);
         vec2 direction   = vec2(cos(vert_angle), sin(vert_angle));
         float distance   = sqrt(dot(vert_offset, vert_offset));

         vec2 vert = distance * direction + rel_focus;

         gl_Position = vec4(
             (vert * from_pixel(sprite_size) + from_pixel(pixel_screen_pos)) * scale,
             0.0f, 1.0f
         );

         int index = flipped != 0 ? flipped_vertex_id() : gl_VertexID;
         if (frame == -1)
             texcoord = TEXCOORD_FROM_ID[index];
         else
             texcoord = frames[frame * 4 + index];
         texcoord.y = 1.0 - texcoord.y;

         cswap_from = color_from(color_swap.x);
         cswap_to   = color_from(color_swap.y);
     }
     ")

    [fragment]
    ("
     in vec4 cswap_from;
     in vec4 cswap_to;

     void main()
     {
        color = texture(tex, texcoord);

        if (approx(color, cswap_from, 0.1))
            color = cswap_to;
     }
     ")
);

