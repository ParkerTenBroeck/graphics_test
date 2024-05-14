use eframe::egui_glow;
use glow::HasContext;

use crate::Texture;

pub struct TileMapContext {
    pub map: TileMap,

    program: glow::Program,
    vertex_array: glow::VertexArray,
    buffer: glow::NativeBuffer,
    pub texture: Texture,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct TileMap {
    pub tiles_x: u16, // the number of tiles actually defined in the array
    pub tiles_y: u16,

    pub tiles_vis_x: u16, // the # of tiles actually visible to the camera
    pub tiles_vis_y: u16,

    pub pan_x: i32, //# of pixels to pan
    pub pan_y: i32,

    pub tiles: Vec<Tile>, // tiles_x * tiles_y long
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Tile {
    pub x: u16,
    pub y: u16,
    pub layer: u8,

    _unused: u8,
    pub attributes: TileAttributes,
}

mycelium_bitfield::bitfield! {
    #[derive(Default, PartialEq, Eq)]
    pub struct TileAttributes<u16> {
        pub const HORIZONTAL: bool;
        pub const VERTICAL: bool;
        pub const ROTATION = 2;
        const _UNUSED = 12;
    }
}

impl TileMapContext {
    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vertex_array);
        gl.delete_buffer(self.buffer);
        self.texture.destroy(gl);
    }

    pub fn new(gl: &glow::Context, texture: Texture) -> Option<Self> {
        let buffer;
        unsafe {
            buffer = gl.create_buffer().unwrap();
        }
        let shader_version = egui_glow::ShaderVersion::get(gl);

        let program;
        let shaders: Vec<_>;
        let vertex_array;
        unsafe {
            program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                return None;
            }

            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("../shaders/tilemap/vertex.vert"),
                include_str!("../shaders/tilemap/fragment.frag"),
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            shaders = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!(
                            "{}",
                            // shader_version.version_declaration(),
                            shader_source
                        ),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
        }

        let mut map = TileMap {
            tiles_x: 30,
            tiles_y: 26,
            tiles_vis_x: 20,
            tiles_vis_y: 15,
            pan_x: 0,
            pan_y: 0,
            tiles: vec![Default::default(); 30 * 26],
        };
        for (index, val) in map.tiles.iter_mut().enumerate() {
            val.x = (index % map.tiles_vis_x as usize) as u16;
            val.y = (index / map.tiles_vis_x as usize) as u16;
        }
        for i in 0..4 {
            for (index, val) in map.tiles
                [((i * map.tiles_x) as usize)..(4 + (i * map.tiles_x) as usize)]
                .iter_mut()
                .enumerate()
            {
                val.x = 5;
                val.y = 6;
                val.attributes.set(TileAttributes::ROTATION, index as u16);

                val.attributes.set(TileAttributes::HORIZONTAL, i & 0b1 >= 1);
                val.attributes.set(TileAttributes::VERTICAL, i & 0b10 >= 1);
            }
        }
        Some(TileMapContext {
            map,
            program,
            vertex_array,
            buffer,
            texture,
        })
    }

    pub fn paint(&self, gl: &glow::Context, zoom: f32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.texture));

            gl.use_program(Some(self.program));
            gl.uniform_1_f32(gl.get_uniform_location(self.program, "zoom").as_ref(), zoom);

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "tiles_x").as_ref(),
                self.map.tiles_x as i32,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "tiles_y").as_ref(),
                self.map.tiles_y as i32,
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "tiles_vis_x")
                    .as_ref(),
                self.map.tiles_vis_x as i32,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "tiles_vis_y")
                    .as_ref(),
                self.map.tiles_vis_y as i32,
            );

            let mut pan_x = self.map.pan_x;
            if pan_x < 0{
                pan_x = self.map.tiles_x as i32 * 8 + pan_x % (self.map.tiles_x as i32 * 8);
            }
            let mut pan_y = self.map.pan_y;
            if pan_y < 0{
                pan_y = self.map.tiles_y as i32 * 8 + pan_y % (self.map.tiles_y as i32 * 8);
            }

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_x").as_ref(),
                pan_x,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_y").as_ref(),
                pan_y,
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "map_width").as_ref(),
                self.texture.width,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "map_height").as_ref(),
                self.texture.height,
            );

            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.buffer));
            gl.buffer_data_u8_slice(
                glow::SHADER_STORAGE_BUFFER,
                std::slice::from_raw_parts(
                    self.map.tiles.as_ptr().cast(),
                    self.map.tiles.len() * std::mem::size_of::<Tile>(),
                ),
                glow::DYNAMIC_DRAW,
            );
            // gl.buffer_sub_data_u8_slice(target, offset, src_data)
            gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 3, Some(self.buffer));
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);

            gl.bind_vertex_array(Some(self.vertex_array));

            gl.draw_arrays(
                glow::TRIANGLES,
                0,
                ((self.map.tiles_vis_x as i32 + 1) * (self.map.tiles_vis_y as i32 + 1)) * 6,
            );
        }
    }
}
