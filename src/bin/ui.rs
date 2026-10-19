//! Rust UI window (egui). Tuxedo theme matching the suite layout.
//!
//! cargo run --features ui --bin ui
//!
//! Top mode tabs, left tool catalog, center canvas,
//! right layers/masks/properties, bottom Maya shelf and status.

use eframe::egui;
use obs_rust_bitstream::{LayerPixels, LayerStack, LayerStackPixels, MayaShelf, MayaShelves, Psd, PsGroup, PsToolbar, ToolDispatch};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tuxedo Edit",
        options,
        Box::new(|_cc| Ok(Box::new(TuxedoApp::default()))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Photoshop,
    AfterEffects,
    Maya,
    Obs,
}

struct TuxedoApp {
    mode: Mode,
    tool: String,
    expanded: Option<PsGroup>,
    maya_shelf: MayaShelf,
    zoom: f32,
    layers: LayerStack,
    selected_layer: String,
    doc: Option<Psd>,
    streaming: bool,
    recording: bool,
    canvas_texture: Option<egui::TextureHandle>,
    last_action: String,
    density: f32,
    feather: f32,
    opacity: f32,
    strokes: Vec<(u32, u32, &'static str)>,
    time: f32,
    keys: Vec<f32>,
    maya_objects: Vec<(String, f32, f32, f32, bool)>,
    selected_object: String,
    pixel_stack: LayerStackPixels,
}

impl Default for TuxedoApp {
    fn default() -> Self {
        let mut layers = LayerStack::new();
        let _ = layers.add("Mountain Spire");
        let _ = layers.add("Mist Overlay");
        let _ = layers.add("Valley Lights");
        let _ = layers.add("Rock Foreground");
        Self {
            mode: Mode::Photoshop,
            tool: "Brush Tool".to_string(),
            expanded: Some(PsGroup::Paint),
            maya_shelf: MayaShelf::Modeling,
            zoom: 27.3,
            layers,
            selected_layer: "Mountain Spire".to_string(),
            doc: Psd::load(std::path::Path::new("assets/mountain_spire.psd")),
            streaming: true,
            recording: true,
            canvas_texture: None,
            last_action: String::new(),
            density: 100.0,
            feather: 0.0,
            opacity: 100.0,
            strokes: Vec::new(),
            time: 0.0,
            keys: vec![0.0, 1.0, 2.5],
            maya_objects: vec![
                ("joint1".to_string(), 0.0, 0.0, 0.0, true),
                ("mesh1".to_string(), 1.0, 0.0, 0.0, true),
            ],
            selected_object: "joint1".to_string(),
            pixel_stack: {
                let mut s = LayerStackPixels::new();
                s.add(LayerPixels::new("Mountain Spire", "assets/mountain_spire.jpg"));
                s.add(LayerPixels::new("Mist Overlay", "assets/mountain_spire.jpg"));
                s
            },
        }
    }
}

impl eframe::App for TuxedoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg = egui::Color32::from_rgb(10, 10, 10);
        let fg = egui::Color32::from_rgb(245, 245, 245);
        let gold = egui::Color32::from_rgb(201, 168, 76);
        let muted = egui::Color32::from_rgb(107, 107, 107);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, bg);

            // Top bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("◆ TUXEDO EDIT").color(gold).size(16.0));
                ui.label(egui::RichText::new("Professional Creative Suite").color(muted).size(10.0));
                ui.separator();
                for (m, label) in [
                    (Mode::Photoshop, "PHOTOSHOP"),
                    (Mode::AfterEffects, "AFTER EFFECTS"),
                    (Mode::Maya, "MAYA"),
                    (Mode::Obs, "OBS"),
                ] {
                    let color = if self.mode == m { gold } else { fg };
                    if ui.button(egui::RichText::new(label).color(color)).clicked() {
                        self.mode = m;
                    }
                }
                ui.separator();
                ui.label(egui::RichText::new("Pro User").color(fg));
                ui.label(egui::RichText::new("LICENSED").color(gold));
            });

            ui.separator();

            ui.horizontal(|ui| {
                // Left: tool catalog
                ui.vertical(|ui| {
                    ui.set_width(200.0);
                    ui.label(egui::RichText::new("◆ PHOTOSHOP").color(gold));
                    for group in PsToolbar::groups() {
                        let open = self.expanded == Some(*group);
                        let label = format!("{} {}", if open { "▼" } else { "▶" }, group.label().to_uppercase());
                        if ui.button(egui::RichText::new(label).color(gold)).clicked() {
                            self.expanded = if open { None } else { Some(*group) };
                        }
                        if open {
                            for tool in PsToolbar::in_group(*group) {
                                let text = format!("  {}  {}", tool.name, tool.hotkey);
                                let color = if self.tool == tool.name { gold } else { fg };
                                if ui.button(egui::RichText::new(text).color(color)).clicked() {
                                    self.tool = tool.name.to_string();
                                }
                            }
                        }
                    }
                });

                ui.separator();

                // Center: canvas
                ui.vertical(|ui| {
                    // Canvas with real pixels
                    if self.canvas_texture.is_none() {
                        if let Ok(img) = image::open("assets/mountain_spire.jpg") {
                            let rgba = img.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                            self.canvas_texture = Some(ctx.load_texture("canvas", color, egui::TextureOptions::LINEAR));
                        }
                    }
                    if let Some(tex) = &self.canvas_texture {
                        let size = tex.size_vec2();
                        let aspect = size.y / size.x;
                        let w = 700.0;
                        let h = w * aspect;
                        let factor = {
                            let vis = self.pixel_stack.visible();
                            if vis.is_empty() { 0.0 } else { vis.len() as f32 / self.pixel_stack.layers.len() as f32 }
                        };
                        ui.image((tex.id(), egui::vec2(w, h)));
                        if factor < 1.0 {
                            let alpha = ((1.0 - factor) * 200.0) as u8;
                            ui.painter().rect_filled(ui.min_rect(), 0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, alpha));
                        }
                        let origin = ui.min_rect().min;
                        for (sx, sy, kind) in &self.strokes {
                            let p = origin + egui::vec2(*sx as f32, *sy as f32);
                            let color = match *kind {
                                "select" => egui::Color32::from_rgb(100, 150, 255),
                                "crop" => egui::Color32::from_rgb(255, 200, 100),
                                "heal" => egui::Color32::from_rgb(100, 255, 150),
                                _ => gold,
                            };
                            ui.painter().circle_filled(p, 4.0, color);
                        }
                        let resp = ui.interact(ui.min_rect(), ui.id().with("canvas"), egui::Sense::click_and_drag());
                        if resp.clicked() || resp.dragged() {
                            if let Some(pos) = resp.interact_pointer_pos() {
                                let rel = pos - ui.min_rect().min;
                                let x = (rel.x.max(0.0) as u32).min(w as u32);
                                let y = (rel.y.max(0.0) as u32).min(h as u32);
                                let action = ToolDispatch::dispatch(&self.tool, x, y);
                                self.last_action = format!("{:?}", action);
                                match action {
                                    obs_rust_bitstream::CanvasAction::Paint { .. } => self.strokes.push((x, y, "paint")),
                                    obs_rust_bitstream::CanvasAction::Select { .. } => self.strokes.push((x, y, "select")),
                                    obs_rust_bitstream::CanvasAction::Crop { .. } => self.strokes.push((x, y, "crop")),
                                    obs_rust_bitstream::CanvasAction::Heal { .. } => self.strokes.push((x, y, "heal")),
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("No image").color(muted));
                    }

                    ui.separator();
                    // Maya shelf
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("MAYA").color(gold));
                        for shelf in MayaShelves::shelves() {
                            let active = self.maya_shelf == *shelf;
                            let color = if active { gold } else { fg };
                            if ui.button(egui::RichText::new(shelf.label()).color(color)).clicked() {
                                self.maya_shelf = *shelf;
                            }
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        for btn in MayaShelves::on_shelf(self.maya_shelf) {
                            if ui.button(egui::RichText::new(btn.name).color(fg)).clicked() {
                                self.tool = btn.name.to_string();
                            }
                        }
                    });
                });

                ui.separator();

                // Right: mode-specific panel
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    match self.mode {
                        Mode::Photoshop => {
                            ui.label(egui::RichText::new("◆ LAYERS").color(gold));
                            if let Some(doc) = &self.doc {
                                ui.label(egui::RichText::new(format!("mountain_spire  {}×{}", doc.width, doc.height)).color(muted));
                                let layer_info: Vec<(String, bool, u8)> = doc.layers.iter().map(|l| (l.name.clone(), l.visible, l.opacity)).collect();
                                for (name, visible, opacity) in layer_info {
                                    let eye = if visible { "◉" } else { "○" };
                                    let color = if self.selected_layer == name { gold } else { fg };
                                    ui.horizontal(|ui| {
                                        if ui.button(egui::RichText::new(eye).color(color)).clicked() {
                                            if let Some(doc) = &mut self.doc {
                                                doc.toggle(&name);
                                            }
                                        }
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                                        if let Some(tex) = &self.canvas_texture {
                                            ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                        } else {
                                            ui.painter().rect_filled(rect, 2.0, gold);
                                        }
                                        if ui.button(egui::RichText::new(format!("{}  {}", name, opacity)).color(color)).clicked() {
                                            self.selected_layer = name.clone();
                                        }
                                    });
                                }
                            } else {
                                ui.label(egui::RichText::new("No document").color(muted));
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("◆ MASKS").color(gold));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Density").color(fg));
                                ui.add(egui::Slider::new(&mut self.density, 0.0..=100.0).suffix("%"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Feather").color(fg));
                                ui.add(egui::Slider::new(&mut self.feather, 0.0..=50.0).suffix(" px"));
                            });
                            ui.separator();
                            ui.label(egui::RichText::new("◆ PROPERTIES").color(gold));
                            ui.label(egui::RichText::new(format!("Selected: {}", self.selected_layer)).color(fg));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Opacity").color(fg));
                                if ui.add(egui::Slider::new(&mut self.opacity, 0.0..=100.0).suffix("%")).changed() {
                                    if let Some(doc) = &mut self.doc {
                                        doc.set_opacity(&self.selected_layer, (self.opacity * 2.55) as u8);
                                    }
                                }
                            });
                            if ui.button(egui::RichText::new("Save PSD").color(gold)).clicked() {
                                if let Some(doc) = &mut self.doc {
                                    doc.set_opacity(&self.selected_layer, (self.opacity * 2.55) as u8);
                                    let _ = doc.save(std::path::Path::new("assets/mountain_spire.psd"));
                                    self.last_action = format!("Saved d={:.0} f={:.0}", self.density, self.feather);
                                }
                            }
                        }
                        Mode::AfterEffects => {
                            ui.label(egui::RichText::new("◆ TIMELINE").color(gold));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Time").color(fg));
                                ui.add(egui::Slider::new(&mut self.time, 0.0..=5.0).suffix(" s"));
                            });
                            ui.label(egui::RichText::new(format!("{:.2}s", self.time)).color(muted));
                            ui.separator();
                            ui.label(egui::RichText::new("KEYFRAMES").color(gold));
                            for k in &self.keys {
                                let mark = if (self.time - k).abs() < 0.1 { "◆" } else { "◇" };
                                ui.label(egui::RichText::new(format!("{}  {:.1}s", mark, k)).color(fg));
                            }
                            if ui.button(egui::RichText::new("Add Key").color(gold)).clicked() {
                                self.keys.push(self.time);
                            }
                        }
                        Mode::Maya => {
                            ui.label(egui::RichText::new("◆ OUTLINER").color(gold));
                            for (name, _, _, _, vis) in &self.maya_objects {
                                let eye = if *vis { "◉" } else { "○" };
                                let color = if self.selected_object == *name { gold } else { fg };
                                if ui.button(egui::RichText::new(format!("{}  {}", eye, name)).color(color)).clicked() {
                                    self.selected_object = name.clone();
                                }
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("◆ ATTRIBUTES").color(gold));
                            if let Some(obj) = self.maya_objects.iter_mut().find(|o| o.0 == self.selected_object) {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("X").color(fg));
                                    ui.add(egui::Slider::new(&mut obj.1, -10.0..=10.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Y").color(fg));
                                    ui.add(egui::Slider::new(&mut obj.2, -10.0..=10.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Z").color(fg));
                                    ui.add(egui::Slider::new(&mut obj.3, -10.0..=10.0));
                                });
                                if ui.button(egui::RichText::new("Toggle Visible").color(gold)).clicked() {
                                    obj.4 = !obj.4;
                                }
                            }
                        }
                        Mode::Obs => {
                            ui.label(egui::RichText::new("◆ STREAM").color(gold));
                            let stream_color = if self.streaming { egui::Color32::from_rgb(0, 200, 0) } else { muted };
                            if ui.button(egui::RichText::new("● Streaming").color(stream_color)).clicked() {
                                self.streaming = !self.streaming;
                            }
                            let rec_color = if self.recording { egui::Color32::from_rgb(200, 0, 0) } else { muted };
                            if ui.button(egui::RichText::new("● Recording").color(rec_color)).clicked() {
                                self.recording = !self.recording;
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("◆ STATUS").color(gold));
                            let stream_line = if self.streaming { "stream=live" } else { "stream=idle" };
                            let rec_line = if self.recording { "record=on" } else { "record=off" };
                            ui.label(egui::RichText::new(stream_line).color(fg));
                            ui.label(egui::RichText::new(rec_line).color(fg));
                            ui.label(egui::RichText::new("output=rtmp").color(muted));
                        }
                    }
                });
            });

            ui.separator();

            // Bottom status
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Tool: {}  B", self.tool)).color(fg));
                if !self.last_action.is_empty() {
                    ui.label(egui::RichText::new(&self.last_action).color(gold));
                }
                ui.separator();
                ui.label(egui::RichText::new(format!("Zoom: {:.1}%", self.zoom)).color(fg));
                ui.separator();
                ui.label(egui::RichText::new("Resolution: 5760 × 3240 px").color(fg));
                ui.separator();
                ui.label(egui::RichText::new("Document: mountain_spire_v2.psd").color(fg));
                ui.separator();
                let stream_color = if self.streaming { egui::Color32::from_rgb(0, 200, 0) } else { muted };
                ui.label(egui::RichText::new("● Streaming: Live").color(stream_color));
                let rec_color = if self.recording { egui::Color32::from_rgb(200, 0, 0) } else { muted };
                ui.label(egui::RichText::new("● REC: 00:47:32").color(rec_color));
            });
        });
    }
}
