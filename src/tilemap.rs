use eframe::egui_glow;
use glow::HasContext;

use crate::resources::{ResourceManager, Texture};

pub struct TileMapContext {
    pub map: TileMap,

    program: glow::Program,
    vertex_array: glow::VertexArray,
    buffer: glow::Buffer,
    last_buffer_size: usize,
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

impl TileMap {
    pub fn recalc(&mut self) {
        self.tiles =  vec![Default::default(); (self.tiles_x * self.tiles_y) as usize];
        for (index, val) in self.tiles.iter_mut().enumerate() {
            val.x = (index % self.tiles_x as usize) as u16;
            val.y = (index / self.tiles_x as usize) as u16;
            val.layer = 50;
        }
        // for y in 0..4.min(self.tiles_y) {
        //     for x in 0..4.min(self.tiles_x){
        //         let val = &mut self.tiles[(x + y * self.tiles_x) as usize];
        //         val.x = 5;
        //         val.y = 6;
        //         val.attributes.set(TileAttributes::ROTATION, x as u16);

        //         val.attributes.set(TileAttributes::HORIZONTAL, y & 0b1 >= 1);
        //         val.attributes.set(TileAttributes::VERTICAL, y & 0b10 >= 1);
        //     }
        // }
    }
}

impl TileMapContext {
    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_vertex_array(self.vertex_array);
        gl.delete_buffer(self.buffer);
    }

    pub fn new(gl: &glow::Context, resources: &mut ResourceManager, texture: Texture) -> Option<Self> {
        let buffer;
        unsafe {
            buffer = gl.create_buffer().unwrap();
        }

        let program;
        let vertex_array;
        unsafe {
            program = resources.get_program(gl, "tilemap", &[
                (crate::resources::ProgramKind::Vertex, include_str!("../shaders/tilemap/vertex.vert")),
                (crate::resources::ProgramKind::Fragment, include_str!("../shaders/tilemap/fragment.frag"))
            ])?;

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
            tiles: Vec::new(),
        };
        map.recalc();
        Some(TileMapContext {
            map,
            program,
            vertex_array,
            buffer,
            texture,
            last_buffer_size: 0
        })
    }

    pub fn paint(&mut self, gl: &glow::Context, zoom: f32) {
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

            {
                let raw_data = std::slice::from_raw_parts(
                    self.map.tiles.as_ptr().cast(),
                    self.map.tiles.len() * std::mem::size_of::<Tile>() / std::mem::size_of::<i32>(),
                );

                gl.uniform_2_i32_slice(
                    gl.get_uniform_location(self.program, "tiles").as_ref(),
                    raw_data
                );
            }
            // gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.buffer));
            // {
            //     let raw_data = std::slice::from_raw_parts(
            //         self.map.tiles.as_ptr().cast(),
            //         self.map.tiles.len() * std::mem::size_of::<Tile>(),
            //     );
            //     if raw_data.len() <= self.last_buffer_size{
            //         gl.buffer_sub_data_u8_slice(
            //             glow::SHADER_STORAGE_BUFFER,
            //             0,
            //             raw_data,
            //         );
            //     }else{
            //         gl.buffer_data_u8_slice(
            //             glow::SHADER_STORAGE_BUFFER,
            //             raw_data,
            //             glow::DYNAMIC_DRAW,
            //         );
            //         self.last_buffer_size = raw_data.len();
            //     }
            // }
            // // gl.buffer_sub_data_u8_slice(target, offset, src_data)
            // gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 3, Some(self.buffer));
            // gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);

            gl.bind_vertex_array(Some(self.vertex_array));

            gl.draw_arrays(
                glow::TRIANGLES,
                0,
                (self.map.tiles_vis_x as i32 + 1) * (self.map.tiles_vis_y as i32 + 1) * 6,
            );
        }
    }
}


