//! Rust UI window (egui). Tuxedo theme.
//!
//! cargo run --features ui --bin ui
//!
//! Black background, white text, gold accent.
//! Toolbar groups expand and retract. Status shows tool and canvas.

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tuxedo Edit",
        options,
        Box::new(|_cc| Ok(Box::new(TuxedoApp::default()))),
    )
}

struct TuxedoApp {
    tool: String,
    expanded: bool,
    zoom: f32,
}

impl Default for TuxedoApp {
    fn default() -> Self {
        Self {
            tool: "Brush".to_string(),
            expanded: false,
            zoom: 100.0,
        }
    }
}

impl eframe::App for TuxedoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg = egui::Color32::from_rgb(10, 10, 10);
        let fg = egui::Color32::from_rgb(245, 245, 245);
        let gold = egui::Color32::from_rgb(201, 168, 76);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, bg);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("◆ TUXEDO EDIT").color(gold).size(18.0));
                ui.separator();
                ui.label(egui::RichText::new("4K").color(gold));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(180.0);
                    for group in ["SELECT", "CROP", "PAINT", "RETOUCH", "DRAW", "VIEW"] {
                        if ui.button(egui::RichText::new(format!("◆ {}", group)).color(gold)).clicked() {
                            if group == "PAINT" {
                                self.expanded = !self.expanded;
                            }
                        }
                        if group == "PAINT" && self.expanded {
                            for tool in ["Brush", "Eraser", "Clone"] {
                                if ui.button(egui::RichText::new(format!("  {}", tool)).color(fg)).clicked() {
                                    self.tool = tool.to_string();
                                }
                            }
                        }
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Canvas").color(fg));
                    let canvas = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(640.0, 360.0));
                    ui.painter().rect_stroke(canvas, 0.0, egui::Stroke::new(1.0, gold));
                    ui.allocate_space(egui::vec2(640.0, 360.0));
                });
            });

            ui.separator();
            ui.label(
                egui::RichText::new(format!("◆ {}  ·  {:.0}%  ·  3840×2160", self.tool.to_uppercase(), self.zoom))
                    .color(fg),
            );
        });
    }
}
