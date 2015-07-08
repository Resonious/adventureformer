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
    ccfft crattlecrute_front_foot: SpriteType2Color2 [9][90;90] "assets/crattlecrute/front-foot.png"
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

     // distance from vertices to center
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
         return vec4(
             float((color & 0xFF000000) >> 24) / 255.0,
             float((color & 0x00FF0000) >> 16) / 255.0,
             float((color & 0x0000FF00) >>  8) / 255.0,
             float(color & 0x000000FF)         / 255.0
         );
     }

     void main()
     {
         vec2 pixel_screen_pos = (position - cam_pos) * 2.0;

         ivec2 ivert = ivec2(vertex_pos) - ivec2(1, 1);
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

     bool approx(vec4 a, vec4 b, float alpha)
     {
         return abs(a.x - b.x) <= alpha &&
                abs(a.y - b.y) <= alpha &&
                abs(a.z - b.z) <= alpha &&
                abs(a.w - b.w) <= alpha;
     }

     void main()
     {
        color = texture(tex, texcoord);

        if (approx(color, cswap1_from, 0.1))
            color = cswap1_to;
        else if (approx(color, cswap2_from, 0.1))
            color = cswap2_to;
     }
     ")
);

