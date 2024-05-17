#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_explicit_attrib_location : enable

const ivec2 verts[4] = ivec2[4](
    ivec2(0, 0),
    ivec2(1, 0),
    ivec2(1, 1),
    ivec2(0, 1)
);
const ivec2 uvs[6] = ivec2[6](
    ivec2(0, 0),
    ivec2(1, 0),
    ivec2(1, 1),

    ivec2(0, 1),
    ivec2(0, 0),
    ivec2(1, 1)
);
out vec2 uv;
uniform float zoom;

uniform int map_width; 
uniform int map_height;

uniform int screen_px_x;
uniform int screen_px_y;

uniform int pan_x;
uniform int pan_y;





struct Sprite
{
    int pos;
    int attributes;
};
layout (location = 2) in ivec2 spriteData;

// layout(std430, binding = 2) buffer spriteBuf
// {
//   Sprite sprites[];
// };

void main() {


    Sprite sprite;
    sprite.pos = spriteData.x;
    sprite.attributes = spriteData.y;

    int sprite_x = sprite.pos & 0xFFFF;
    int sprite_y = (sprite.pos >> 16) & 0xFFFF;


    int uv_x = ((sprite.attributes) & 0xFF)*8;
    int uv_y = ((sprite.attributes>>8) & 0xFF)*8;

    int layer = (sprite.attributes>>16) & 0xFF;


    int flip_h = (sprite.attributes>>(24+0)) & 1;
    int flip_v = (sprite.attributes>>(24+1)) & 1;

    int rotate = (sprite.attributes>>(24+2)) & 3;

    // size in px (8, 16, 24, 32)
    int x_size = ((sprite.attributes>>(24+4)) & 3)*8 + 8;
    int y_size = ((sprite.attributes>>(24+6)) & 3)*8 + 8;



    int index = gl_VertexID % 6;

    // translate the tilemap pixel coord into uv coords
    ivec2 tuv = uvs[index]; 

    // removes rounding artifacts by squizing in each corner by epsilon* (random value I found that works)
    uv.x = float(tuv.x) * -4.20e-07 + float(1 - tuv.x) * 4.20e-07;
    uv.y = float(tuv.y) * -4.20e-07 + float(1 - tuv.y) * 4.20e-07;

    tuv.x *= x_size;
    tuv.y *= y_size;
    tuv.x += uv_x;
    tuv.y += uv_y;

    uv.x += float(tuv.x) / float(map_width);
    uv.y += float(tuv.y) / float(map_height);


    index += index == 5 ? 1 : 0;
    index += rotate;
    index %= 4;

    ivec2 pos = verts[index];


    pos.x ^= flip_h;
    pos.y ^= flip_v;

    int rbit = rotate & 1;
    pos.x *= y_size * rbit + x_size * (rbit^1);
    pos.y *= x_size * rbit + y_size * (rbit^1);

    pos.x += -pan_x + sprite_x;
    pos.y += -pan_y + sprite_y;

 
    gl_Position = vec4(0.0, 0.0,  float(layer)/255.0, 1.0);
    gl_Position.x = float(pos.x) * 2.0/float(screen_px_x) - 1.0;
    gl_Position.y = float(pos.y) * -2.0/float(screen_px_y) + 1.0;

    gl_Position.x *= zoom;
    gl_Position.y *= zoom;
}
