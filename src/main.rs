pub mod tilemap;
pub mod sprites;
pub mod resources;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 1024.0])
            .with_drag_and_drop(true),

        #[cfg(feature = "wgpu")]
        renderer: eframe::Renderer::Wgpu,

        ..Default::default()
    };
    eframe::run_native(
        "graphics test",
        options,
        Box::new(|cc| Box::new(Custom3d::new(cc).unwrap())),
    )
    .unwrap();
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "graphics_test", // hardcode it
                web_options,
                Box::new(|cc| Box::new(Custom3d::new(cc).unwrap())),
            )
            .await
            .expect("failed to start eframe");
    });
}

use std::{io::Cursor, sync::Arc};

use eframe::egui_glow;
use egui::{mutex::Mutex, Slider, Widget};
use egui_glow::glow;
use resources::ResourceManager;
use sprites::{SpriteAttributes, SpriteMapContext};

use crate::{resources::Texture, tilemap::TileMapContext};

pub struct Custom3d {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    retro_graphics: Arc<Mutex<RetroGraphics>>,
    zoom: f32,
    panx: f32,
    pany: f32,
}

impl Custom3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            retro_graphics: Arc::new(Mutex::new(RetroGraphics::new(gl)?)),
            zoom: 0.0,
            panx: 0.0,
            pany: 0.0,
        })
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui|{
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            
                            let mut lock = self.retro_graphics.lock();
    
                            
                            Slider::new(&mut lock.tile_map.map.tiles_vis_x, 1..=30).text(" vis x").show_value(true).ui(ui);
                            Slider::new(&mut lock.tile_map.map.tiles_vis_y, 1..=30).text(" vis y").show_value(true).ui(ui);
    
                            let mut changed = Slider::new(&mut lock.tile_map.map.tiles_x, 1..=30).text(" x").show_value(true).ui(ui).changed();
                            changed |= Slider::new(&mut lock.tile_map.map.tiles_y, 1..=30).text(" y").show_value(true).ui(ui).changed();
    
                            if changed{
                                lock.tile_map.map.recalc();
                            }
    
                            ui.label(format!("pan x: {}", lock.tile_map.map.pan_x));
                            ui.label(format!("pan y: {}", lock.tile_map.map.pan_y));
                            ui.label(format!("zoom: {}", self.zoom));
                        });

                        ui.add_space(1.0);

                        ui.vertical(|ui|{

                            let mut lock = self.retro_graphics.lock();

                            let item = &mut lock.sprite_map. thing[0];

                            Slider::new(&mut item.x, 0..=256).text(" x").show_value(true).step_by(1.0).ui(ui);
                            Slider::new(&mut item.y, 0..=256).text(" y").show_value(true).step_by(1.0).ui(ui);
    
                            Slider::new(&mut item.tx, 0..=255).text(" uv x").show_value(true).step_by(1.0).ui(ui);
                            Slider::new(&mut item.ty, 0..=255).text(" uv y").show_value(true).step_by(1.0).ui(ui);

                            Slider::new(&mut item.layer, 0..=255).text(" layer").show_value(true).step_by(1.0).ui(ui);
                            
                            let mut tmp = item.attribute.get(SpriteAttributes::HORIZONTAL);
                            ui.checkbox(&mut tmp, "Horizontal Flip");
                            item.attribute.set(SpriteAttributes::HORIZONTAL, tmp);
                            
                            let mut tmp = item.attribute.get(SpriteAttributes::VERTICAL);
                            ui.checkbox(&mut tmp, "Vertical Flip");
                            item.attribute.set(SpriteAttributes::VERTICAL, tmp);

                            let mut tmp = item.attribute.get(SpriteAttributes::ROTATION);
                            Slider::new(&mut tmp, 0..=3).text(" rot").show_value(true).ui(ui);
                            item.attribute.set(SpriteAttributes::ROTATION, tmp);

                            let mut tmp = item.attribute.get(SpriteAttributes::XSIZE);
                            Slider::new(&mut tmp, 0..=3).text(" size x").show_value(true).ui(ui);
                            item.attribute.set(SpriteAttributes::XSIZE, tmp);

                            let mut tmp = item.attribute.get(SpriteAttributes::YSIZE);
                            Slider::new(&mut tmp, 0..=3).text(" size y").show_value(true).step_by(1.0).ui(ui);
                            item.attribute.set(SpriteAttributes::YSIZE, tmp);
                        });
                    });
                    

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
                    ui.label("Drag to pan, Scroll to zoom!");
                });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.retro_graphics.lock().destroy(gl);
        }
    }
}

impl Custom3d {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {

        let area;
        {
            let lock = self.retro_graphics.lock();
            let aspect_py = lock.tile_map.map.tiles_vis_y as f32 / lock.tile_map.map.tiles_vis_x as f32;
            let aspect_px = lock.tile_map.map.tiles_vis_x as f32 / lock.tile_map.map.tiles_vis_y as f32;
            if aspect_py < aspect_px{
                area = egui::Vec2::new(600.0, 600.0 * aspect_py);
            }else{
                area = egui::Vec2::new(600.0 * aspect_px, 600.0);
            }
        }
        let (rect, response) =
            ui.allocate_exact_size(area, egui::Sense::drag());

        self.zoom += ui.input(|io| io.smooth_scroll_delta.y * 0.002);

        self.panx -= response.drag_delta().x / rect.width();
        self.pany -= response.drag_delta().y / rect.height();

        // Clone locals so we can move them into the paint callback:
        let zoom = self.zoom;
        let panx = self.panx;
        let pany = self.pany;
        let rotating_triangle = self.retro_graphics.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle
                .lock()
                .paint(painter.gl(), zoom, panx, pany);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

struct RetroGraphics {
    resources: ResourceManager,
    tile_map: TileMapContext,
    sprite_map: SpriteMapContext,
}

fn sprite_sheet() -> (i32, i32, Vec<u8>) {
    let image = include_bytes!("../res/spritesheet.png");
    let thing = image::load(Cursor::new(image), image::ImageFormat::Png).unwrap();
    let other = thing.to_rgba8();
    (
        thing.width() as i32,
        thing.height() as i32,
        other.into_vec(),
    )
}

impl RetroGraphics {
    fn new(gl: &glow::Context) -> Option<Self> {
        use glow::HasContext as _;
        unsafe{
            // gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        let mut resources = ResourceManager::new();
 

        let texture;
        unsafe {
            let (width, height, pixels) = sprite_sheet();
            let ntexture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(ntexture));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width,
                height,
                glow::NONE as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&pixels),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);

            texture = Texture{
                texture: ntexture,
                width,
                height,
            };
            resources.insert_texture(texture);
        }

        Some(Self {
            tile_map: TileMapContext::new(gl, &mut resources, texture).expect("Failed to create tilemap"),
            sprite_map: SpriteMapContext::new(gl, &mut resources, texture).expect("Failed to create tilemap"),
            resources: ResourceManager::new(),
        })
    }

    fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            self.tile_map.destroy(gl);
            self.sprite_map.destroy(gl);
            self.resources.destroy(gl);
        }
    }

    fn paint(&mut self, gl: &glow::Context, zoom: f32, pan_x: f32, pan_y: f32) {
        use glow::HasContext as _;
        unsafe{
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            // gl.enable(glow::DEPTH_TEST);
            // gl.clear(glow::DEPTH_BUFFER_BIT);
        }

        let zoom = zoom.exp();
        self.tile_map.map.pan_x = (pan_x * 8.0 * self.tile_map.map.tiles_vis_x as f32) as i32;
        self.tile_map.map.pan_y = (pan_y * 8.0 * self.tile_map.map.tiles_vis_y as f32) as i32;
        
        
        self.tile_map.paint(gl, zoom);

        self.sprite_map.paint(gl, zoom, 
            self.tile_map.map.tiles_vis_x as i32 * 8, 
            self.tile_map.map.tiles_vis_y as i32 * 8,
            self.tile_map.map.pan_x,
            self.tile_map.map.pan_y
        );

        
    }
}