//! Rust UI window (egui). Tuxedo theme matching the suite layout.
//!
//! cargo run --features ui --bin ui
//!
//! Top mode tabs, left tool catalog, center canvas,
//! right layers/masks/properties, bottom Maya shelf and status.

use eframe::egui;
use obs_rust_bitstream::{LayerStack, MayaShelf, MayaShelves, Phshp, PsGroup, PsToolbar};

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
    doc: Option<Phshp>,
    streaming: bool,
    recording: bool,
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
            doc: Phshp::load(std::path::Path::new("assets/mountain_spire.phshp")),
            streaming: true,
            recording: true,
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
                    ui.label(egui::RichText::new("PHOTOSHOP").color(gold));
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
                    // Canvas with gradient fill
                    let canvas = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(700.0, 400.0));
                    ui.painter().rect_filled(canvas, 4.0, egui::Color32::from_rgb(20, 18, 15));
                    ui.painter().rect_stroke(canvas, 4.0, egui::Stroke::new(1.0_f32, gold));
                    // Simple mountain silhouette
                    let base = canvas.bottom() - 40.0;
                    let points = vec![
                        canvas.left_top() + egui::vec2(0.0, 200.0),
                        canvas.left_top() + egui::vec2(150.0, 80.0),
                        canvas.left_top() + egui::vec2(250.0, 40.0),
                        canvas.left_top() + egui::vec2(350.0, 60.0),
                        canvas.left_top() + egui::vec2(500.0, 100.0),
                        canvas.left_top() + egui::vec2(700.0, 180.0),
                        canvas.right_bottom(),
                        canvas.left_bottom(),
                    ];
                    ui.painter().add(egui::Shape::convex_polygon(
                        points,
                        egui::Color32::from_rgb(40, 35, 30),
                        egui::Stroke::NONE,
                    ));
                    ui.allocate_space(egui::vec2(700.0, 400.0));

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

                // Right: layers, masks, properties
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    ui.label(egui::RichText::new("LAYERS").color(gold));
                    if let Some(doc) = &self.doc {
                        ui.label(egui::RichText::new(format!("{}  {}×{}", doc.name, doc.width, doc.height)).color(muted));
                        for (name, vis) in &doc.layers {
                            let eye = if *vis { "◉" } else { "○" };
                            let color = if self.selected_layer == *name { gold } else { fg };
                            if ui.button(egui::RichText::new(format!("{}  {}", eye, name)).color(color)).clicked() {
                                self.selected_layer = name.clone();
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("No document").color(muted));
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("MASKS").color(gold));
                    ui.label(egui::RichText::new("Density: 100%  Feather: 0.0 px").color(muted));
                    ui.label(egui::RichText::new("Kind: Grayscale  Invert: Off").color(muted));

                    ui.separator();
                    ui.label(egui::RichText::new("PROPERTIES").color(gold));
                    ui.label(egui::RichText::new("Canvas Size").color(fg));
                    ui.label(egui::RichText::new("Width:  5760 px").color(muted));
                    ui.label(egui::RichText::new("Height: 3240 px").color(muted));
                    ui.label(egui::RichText::new("Resolution: 300 Pixels/Inch").color(muted));
                    ui.label(egui::RichText::new("Color Profile").color(fg));
                    ui.label(egui::RichText::new("Working RGB: Adobe RGB (1998)").color(muted));
                });
            });

            ui.separator();

            // Bottom status
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Tool: {}  B", self.tool)).color(fg));
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
