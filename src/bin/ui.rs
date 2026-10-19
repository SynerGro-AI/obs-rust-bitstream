//! Rust UI window (egui). Tuxedo theme.
//!
//! cargo run --features ui --bin ui
//!
//! Pulls the real PsToolbar and MayaShelves catalogs.

use eframe::egui;
use obs_rust_bitstream::{MayaShelf, MayaShelves, PsGroup, PsToolbar};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
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
    expanded: Option<PsGroup>,
    maya_shelf: MayaShelf,
    zoom: f32,
}

impl Default for TuxedoApp {
    fn default() -> Self {
        Self {
            tool: "Brush".to_string(),
            expanded: Some(PsGroup::Paint),
            maya_shelf: MayaShelf::Modeling,
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
                    ui.set_width(220.0);
                    ui.label(egui::RichText::new("PHOTOSHOP").color(gold));
                    for group in PsToolbar::groups() {
                        let open = self.expanded == Some(*group);
                        let label = format!("{} {}", if open { "▼" } else { "▶" }, group.label());
                        if ui.button(egui::RichText::new(label).color(gold)).clicked() {
                            self.expanded = if open { None } else { Some(*group) };
                        }
                        if open {
                            for tool in PsToolbar::in_group(*group) {
                                let text = format!("  {}  {}", tool.name, tool.hotkey);
                                if ui.button(egui::RichText::new(text).color(fg)).clicked() {
                                    self.tool = tool.name.to_string();
                                }
                            }
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("MAYA SHELVES").color(gold));
                    ui.horizontal(|ui| {
                        for shelf in MayaShelves::shelves() {
                            let active = self.maya_shelf == *shelf;
                            let color = if active { gold } else { fg };
                            if ui.button(egui::RichText::new(shelf.label()).color(color)).clicked() {
                                self.maya_shelf = *shelf;
                            }
                        }
                    });
                    for btn in MayaShelves::on_shelf(self.maya_shelf) {
                        if ui.button(egui::RichText::new(format!("  {}", btn.name)).color(fg)).clicked() {
                            self.tool = btn.name.to_string();
                        }
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Canvas").color(fg));
                    let canvas = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(800.0, 450.0));
                    ui.painter().rect_stroke(canvas, 0.0, egui::Stroke::new(1.0_f32, gold));
                    ui.allocate_space(egui::vec2(800.0, 450.0));
                });
            });

            ui.separator();
            ui.label(
                egui::RichText::new(format!(
                    "◆ {}  ·  {:.0}%  ·  3840×2160",
                    self.tool.to_uppercase(),
                    self.zoom
                ))
                .color(fg),
            );
        });
    }
}
