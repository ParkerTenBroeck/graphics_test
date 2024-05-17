use egui::ahash::{HashMap, HashSet};
use glow::HasContext;

#[derive(Default)]
pub struct ResourceManager{
    programs: HashMap<String, glow::Program>,
    textures: HashSet<Texture>
}

pub enum ProgramKind{
    Vertex,
    Fragment,
}

impl ResourceManager{

    pub fn new() -> Self{
        Self::default()
    }

    pub unsafe fn destroy(&mut self, gl: &glow::Context) {
        for (_, program) in self.programs.drain(){
            gl.delete_program(program);
        }

        for texture in self.textures.drain(){
            gl.delete_texture(texture.texture);
        }
    }

    pub fn insert_texture(&mut self, texture: Texture) {
        self.textures.insert(texture);
    }

    pub fn remove_texture(&mut self, texture: Texture) {
        self.textures.remove(&texture);
    }

    pub fn get_program(&mut self, gl: &glow::Context, name: &str, shader_sources: &[(ProgramKind, &str)]) -> Option<glow::Program>{
        if let Some(program) = self.programs.get(name){
            return Some(*program);
        }
        unsafe{
            let shader_version = eframe::egui_glow::ShaderVersion::get(gl);
            let program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                return None;
            }
    
            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader_type = match shader_type{
                        ProgramKind::Vertex => glow::VERTEX_SHADER,
                        ProgramKind::Fragment => glow::FRAGMENT_SHADER,
                    };
                    let shader = gl
                        .create_shader(shader_type)
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

            assert!(self.programs.insert(name.into(), program).is_none());

            Some(program)
        }


    }
}


#[derive(Clone, Copy, Eq)]
pub struct Texture {
    pub texture: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl std::hash::Hash for Texture{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.texture.hash(state);
    }
}

impl std::cmp::PartialEq for Texture{
    fn eq(&self, other: &Self) -> bool {
        self.texture == other.texture
    }
}

impl Texture {
    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_texture(self.texture);
        }
    }
}