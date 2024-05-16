use eframe::egui_glow;
use glow::HasContext;

use crate::Texture;



#[derive(Clone)]
pub struct SpriteMapContext{
    texture: Texture,
    pub thing: Vec<Sprite>,


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
    pub fn new(gl: &glow::Context, texture: Texture) -> Option<Self>{
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
                include_str!("../shaders/sprite/vertex.vert"),
                include_str!("../shaders/sprite/fragment.frag"),
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
                            "{}\n{}",
                            shader_version.version_declaration(),
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

        Some(Self { texture, thing: vec![
            Sprite{ x: 10, y: 10, tx: 7*2, ty: 3*2, layer: 2, attribute: SpriteAttributes(0b00001010) },
            // Sprite{ x: 10, y: 10, tx: 5, ty: 0, layer: 3, attribute: SpriteAttributes(0b00000000) },
        ], program, vertex_array, buffer, last_buffer_size: 0 })
    }

    pub unsafe fn destroy(&self, gl: &glow::Context){
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vertex_array);
        gl.delete_buffer(self.buffer);
    }
    
    pub fn paint(&mut self, gl: &glow::Context, zoom: f32, screen_px_x: i32, screen_px_y: i32, pan_x: i32, pan_y: i32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.texture));

            gl.use_program(Some(self.program));
            
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "zoom")
                    .as_ref(),
                    zoom,
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
                    screen_px_x
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "screen_px_y")
                    .as_ref(),
                    screen_px_y
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_x")
                    .as_ref(),
                    pan_x
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "pan_y")
                    .as_ref(),
                    pan_y
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
