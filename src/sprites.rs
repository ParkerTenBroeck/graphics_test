use glow::HasContext;

use crate::{resources::{ResourceManager, Texture}, ScreenContext};



#[derive(Clone)]
pub struct SpriteMapContext{
    pub thing: Vec<Sprite>,
    pub pan_x: i32, 
    pub pan_y: i32,


    texture: Texture,
    program: glow::Program,
    vertex_array: glow::VertexArray,
    buffer: glow::Buffer,
    last_buffer_size: usize,
}



#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Sprite {
    pub x: u16,
    pub y: u16,

    pub tx: u8,
    pub ty: u8,

    pub layer: u8,
    pub attribute: SpriteAttributes,
}

mycelium_bitfield::bitfield! {
    #[derive(Default, PartialEq, Eq)]
    pub struct SpriteAttributes<u8> {
        pub const HORIZONTAL: bool;
        pub const VERTICAL: bool;
        pub const ROTATION = 2;
        pub const XSIZE = 2;
        pub const YSIZE = 2;
    }
}


impl SpriteMapContext{
    pub fn new(gl: &glow::Context, resources: &mut ResourceManager, texture: Texture) -> Option<Self>{
        let buffer;
        unsafe {
            buffer = gl.create_buffer().unwrap();
        }

        let program;
        let vertex_array;
        unsafe {
            program = resources.get_program(gl, "sprites", &[
                (crate::resources::ProgramKind::Vertex, include_str!("../shaders/sprite/vertex.vert")),
                (crate::resources::ProgramKind::Fragment, include_str!("../shaders/sprite/fragment.frag"))
            ])?;

            vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
        }

        Some(Self { pan_x: 0, pan_y: 0, thing: vec![
            Sprite{ x: 10, y: 10, tx: 7*2, ty: 3*2, layer: 2, attribute: SpriteAttributes(0b00001010) },
            // Sprite{ x: 10, y: 10, tx: 5, ty: 0, layer: 3, attribute: SpriteAttributes(0b00000000) },
        ], program, vertex_array, buffer, last_buffer_size: 0, texture,  })
    }

    pub unsafe fn destroy(&self, gl: &glow::Context){
        gl.delete_vertex_array(self.vertex_array);
        gl.delete_buffer(self.buffer);
    }
    
    pub fn paint(&mut self, gl: &glow::Context, screen: &ScreenContext) {
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.texture));

            gl.use_program(Some(self.program));
            
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "zoom")
                    .as_ref(),
                    screen.zoom,
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "map_width")
                    .as_ref(),
                    self.texture.width
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "map_height")
                    .as_ref(),
                    self.texture.height
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "screen_px_x")
                    .as_ref(),
                    screen.screen_px_x
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "screen_px_y")
                    .as_ref(),
                    screen.screen_px_y
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_x")
                    .as_ref(),
                    self.pan_x
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_y")
                    .as_ref(),
                    self.pan_y
            );


            gl.bind_vertex_array(Some(self.vertex_array));
            
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            gl.enable_vertex_attrib_array(2);

            {
                let raw_data = std::slice::from_raw_parts(
                    self.thing.as_ptr().cast(),
                    self.thing.len() * std::mem::size_of::<Sprite>(),
                );
                if raw_data.len() <= self.last_buffer_size{
                    gl.buffer_sub_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        0,
                        raw_data,
                    );
                }else{
                    gl.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        raw_data,
                        glow::DYNAMIC_DRAW,
                    );
                    self.last_buffer_size = raw_data.len();
                }
            }
            // gl.buffer_sub_data_u8_slice(target, offset, src_data)
            // gl.bind_buffer_base(glow::ARRAY_BUFFER, 0, Some(self.buffer));
            gl.vertex_attrib_pointer_i32(2, 2, glow::INT, 2*std::mem::size_of::<i32>() as i32, 0);
            gl.vertex_attrib_divisor(2, 1);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);


            gl.draw_arrays_instanced(
                glow::TRIANGLES,
                0,
                6,
                self.thing.len() as i32,
            );
        }
    }
}
