#version 430

const vec2 verts[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0)
);
const ivec2 uvs[6] = ivec2[](
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

uniform int tiles_vis_x;
uniform int tiles_vis_y;

uniform int tiles_x;
uniform int tiles_y;

uniform int pan_x;
uniform int pan_y;


struct Tile
{
    int pos;
    int attributes;
};

layout(std430, binding = 3) buffer tileBuf
{
  Tile tiles[];
};

void main() {

    int tile_id = gl_VertexID / 6;

    int tile_x = tile_id % (tiles_vis_x+1);
    int tile_y = tile_id / (tiles_vis_x+1);

    tile_id = (tile_x + pan_x / 8) % tiles_x + ((tile_y + pan_y / 8) % tiles_y) * tiles_x;

    Tile tile = tiles[tile_id];

    int layer = tile.attributes & 0xFF;
    // values multipied by 2
    int flip_h = (tile.attributes>>15) & 2;
    int flip_v = (tile.attributes>>16) & 2;

    int rotate = (tile.attributes>>18) & 3;


    int index = gl_VertexID % 6;

    // translate the tilemap pixel coord into uv coords
    ivec2 tuv = uvs[index]; 

    // removes rounding artifacts by squizing in each corner by epsilon* (random value I found that works)
    uv.x = tuv.x * -4.20e-07 + (1 - tuv.x) * 4.20e-07;
    uv.y = tuv.y * -4.20e-07 + (1 - tuv.y) * 4.20e-07;

    tuv.x *= 8;
    tuv.y *= 8;
    tuv.x += (tile.pos & 0xFFFF) * 8;
    tuv.y += ((tile.pos >> 16) & 0xFFFF) * 8;

    uv.x += tuv.x / float(map_width);
    uv.y += tuv.y / float(map_height);


    index += index == 5 ? 1 : 0;
    index += rotate;
    index %= 4;
    gl_Position = vec4(verts[index],  layer/255.0, 1.0);

    gl_Position.x *= -(flip_h-1);
    gl_Position.y *= -(flip_v-1);
    // gl_Position.x *= cos(u_angle);


    gl_Position.x /= tiles_vis_x;
    gl_Position.x -= 1 - 1.0/tiles_vis_x;
    gl_Position.x += tile_x * 2.0/tiles_vis_x;
    gl_Position.x -= (pan_x%8)/8.0 * 2.0/tiles_vis_x;

    gl_Position.y /= -tiles_vis_y;
    gl_Position.y += 1 - 1.0/tiles_vis_y;
    gl_Position.y -= tile_y * 2.0/tiles_vis_y;
    gl_Position.y += (pan_y%8)/8.0 * 2.0/tiles_vis_y;

    gl_Position.x *= zoom;
    gl_Position.y *= zoom;
}