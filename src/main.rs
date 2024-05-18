pub mod resources;
pub mod sprites;
pub mod tilemap;

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
use egui::{mutex::Mutex, ComboBox, Slider, Widget};
use egui_glow::glow;
use resources::ResourceManager;
use sprites::SpriteMapContext;

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
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut lock = self.retro_graphics.lock();
                    ui.vertical(|ui| {
                        ui.label(format!("zoom: {}", lock.screen.zoom));

                        Slider::new(&mut lock.screen.screen_px_x, 0..=256)
                            .text(" pixels x")
                            .show_value(true)
                            .step_by(8.0)
                            .ui(ui);
                        Slider::new(&mut lock.screen.screen_px_y, 0..=256)
                            .text(" pixels y")
                            .show_value(true)
                            .step_by(8.0)
                            .ui(ui);
                    });
                    for (index, item) in lock.layers.iter_mut().enumerate() {
                        ui.add_space(1.0);

                        ui.vertical(|ui| match item {
                            Layer::Sprite(sprites) => {
                                ComboBox::new(index, "Sprite").show_ui(ui, |ui| {
                                    for i in 0..sprites.thing.len() {
                                        ui.label(format!("{i}"));
                                    }
                                });
                            }
                            Layer::TileMap(tilemap) => {
                                let mut changed = Slider::new(&mut tilemap.map.tiles_x, 1..=30)
                                    .text(" tiles x")
                                    .show_value(true)
                                    .ui(ui)
                                    .changed();
                                changed |= Slider::new(&mut tilemap.map.tiles_y, 1..=30)
                                    .text(" tiles y")
                                    .show_value(true)
                                    .ui(ui)
                                    .changed();

                                if changed {
                                    tilemap.map.recalc();
                                }

                                ui.label(format!("pan x: {}", tilemap.map.pan_x));
                                ui.label(format!("pan y: {}", tilemap.map.pan_y));
                            }
                            Layer::Bitmap() => todo!(),
                            Layer::Effect() => todo!(),
                        });
                    }

                    ui.vertical(|ui| {
                        // let item = &mut lock.sprite_map. thing[0];

                        // Slider::new(&mut item.x, 0..=256).text(" x").show_value(true).step_by(1.0).ui(ui);
                        // Slider::new(&mut item.y, 0..=256).text(" y").show_value(true).step_by(1.0).ui(ui);

                        // Slider::new(&mut item.tx, 0..=255).text(" uv x").show_value(true).step_by(1.0).ui(ui);
                        // Slider::new(&mut item.ty, 0..=255).text(" uv y").show_value(true).step_by(1.0).ui(ui);

                        // Slider::new(&mut item.layer, 0..=255).text(" layer").show_value(true).step_by(1.0).ui(ui);

                        // let mut tmp = item.attribute.get(SpriteAttributes::HORIZONTAL);
                        // ui.checkbox(&mut tmp, "Horizontal Flip");
                        // item.attribute.set(SpriteAttributes::HORIZONTAL, tmp);

                        // let mut tmp = item.attribute.get(SpriteAttributes::VERTICAL);
                        // ui.checkbox(&mut tmp, "Vertical Flip");
                        // item.attribute.set(SpriteAttributes::VERTICAL, tmp);

                        // let mut tmp = item.attribute.get(SpriteAttributes::ROTATION);
                        // Slider::new(&mut tmp, 0..=3).text(" rot").show_value(true).ui(ui);
                        // item.attribute.set(SpriteAttributes::ROTATION, tmp);

                        // let mut tmp = item.attribute.get(SpriteAttributes::XSIZE);
                        // Slider::new(&mut tmp, 0..=3).text(" size x").show_value(true).ui(ui);
                        // item.attribute.set(SpriteAttributes::XSIZE, tmp);

                        // let mut tmp = item.attribute.get(SpriteAttributes::YSIZE);
                        // Slider::new(&mut tmp, 0..=3).text(" size y").show_value(true).step_by(1.0).ui(ui);
                        // item.attribute.set(SpriteAttributes::YSIZE, tmp);
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
            let aspect_py = lock.screen.screen_px_y as f32 / lock.screen.screen_px_x as f32;
            let aspect_px = lock.screen.screen_px_x as f32 / lock.screen.screen_px_y as f32;
            if aspect_py < aspect_px {
                area = egui::Vec2::new(600.0, 600.0 * aspect_py);
            } else {
                area = egui::Vec2::new(600.0 * aspect_px, 600.0);
            }
        }
        let (rect, response) = ui.allocate_exact_size(area, egui::Sense::drag());

        self.zoom += ui.input(|io| io.smooth_scroll_delta.y * 0.002);

        self.panx -= response.drag_delta().x / rect.width();
        self.pany -= response.drag_delta().y / rect.height();

        // Clone locals so we can move them into the paint callback:
        let zoom = self.zoom;
        let pan_x = self.panx;
        let pan_y = self.pany;
        let rotating_triangle = self.retro_graphics.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            let mut lock = rotating_triangle.lock();

            lock.screen.zoom = zoom.exp();
            let pan_x = (pan_x * lock.screen.screen_px_x as f32) as i32;
            let pan_y = (pan_y * lock.screen.screen_px_y as f32) as i32;

            for layer in lock.layers.iter_mut() {
                match layer {
                    Layer::Sprite(l) => {
                        l.pan_x = pan_x;
                        l.pan_y = pan_y;
                    }
                    Layer::TileMap(l) => {
                        l.map.pan_x = pan_x;
                        l.map.pan_y = pan_y;
                    }
                    Layer::Bitmap() => {}
                    Layer::Effect() => {}
                }
            }

            lock.paint(painter.gl());
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

#[allow(unused)]
enum Layer {
    Sprite(SpriteMapContext),
    TileMap(TileMapContext),
    Bitmap(),
    Effect(),
}

impl Layer {
    pub unsafe fn destroy(&mut self, gl: &glow::Context) {
        match self {
            Layer::Sprite(l) => l.destroy(gl),
            Layer::TileMap(l) => l.destroy(gl),
            Layer::Bitmap() => todo!(),
            Layer::Effect() => todo!(),
        }
    }

    pub unsafe fn paint(&mut self, gl: &glow::Context, screen: &ScreenContext) {
        match self {
            Layer::Sprite(l) => l.paint(gl, screen),
            Layer::TileMap(l) => l.paint(gl, screen),
            Layer::Bitmap() => todo!(),
            Layer::Effect() => todo!(),
        }
    }
}

pub struct ScreenContext {
    screen_px_x: i32,
    screen_px_y: i32,
    zoom: f32,
}

struct RetroGraphics {
    resources: ResourceManager,
    screen: ScreenContext,
    layers: Vec<Layer>,
    // tile_map: TileMapContext,
    // sprite_map: SpriteMapContext,
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
        unsafe {
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

            texture = Texture {
                texture: ntexture,
                width,
                height,
            };
            resources.insert_texture(texture);
        }

        Some(Self {
            layers: vec![
                Layer::Sprite(
                    SpriteMapContext::new(gl, &mut resources, texture)
                        .expect("Failed to create tilemap"),
                ),
                Layer::TileMap(
                    TileMapContext::new(gl, &mut resources, texture)
                        .expect("Failed to create tilemap"),
                ),
            ],
            screen: ScreenContext {
                screen_px_x: 256,
                screen_px_y: 224,
                zoom: 1.0,
            },
            resources: ResourceManager::new(),
        })
    }

    fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            for layer in &mut self.layers {
                layer.destroy(gl)
            }
            // self.tile_map.destroy(gl);
            // self.sprite_map.destroy(gl);
            self.resources.destroy(gl);
        }
    }

    fn paint(&mut self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            // gl.enable(glow::DEPTH_TEST);
            // gl.clear(glow::DEPTH_BUFFER_BIT);
        }
        for layer in self.layers.iter_mut().rev() {
            unsafe {
                layer.paint(gl, &self.screen);
            }
        }

        // for layer in self.
        // self.tile_map.paint(gl, zoom);

        // self.sprite_map.paint(gl, zoom,
        //     self.tile_map.map.tiles_vis_x as i32 * 8,
        //     self.tile_map.map.tiles_vis_y as i32 * 8,
        //     self.tile_map.map.pan_x,
        //     self.tile_map.map.pan_y
        // );
    }
}
