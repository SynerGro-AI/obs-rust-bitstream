//! Composite mode: layers with keyframed opacity over time.

use crate::theme::{self, Palette, R_MD, R_SM, SP_MD, SP_SM, SP_XS};
use crate::timeline::{self, Clock, Track};
use crate::resolve_asset;
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use obs_rust_bitstream::ui_canvas::{composite, format_clock, BlendMode, ImageLayer, KeyTrack, Raster, ViewTransform};

pub const PW: u32 = 584;
pub const PH: u32 = 392;

pub struct CompLayer {
    pub layer: ImageLayer,
    pub opacity: KeyTrack,
}

pub struct CompState {
    pub layers: Vec<CompLayer>,
    pub selected: usize,
    pub clock: Clock,
    tex: Option<egui::TextureHandle>,
    last: Option<(f32, Vec<f32>, Vec<bool>, Vec<BlendMode>)>,
}

fn load_preview(path: &str) -> Option<Raster> {
    let img = image::open(resolve_asset(path)).ok()?;
    let img = img.resize_exact(PW, PH, image::imageops::FilterType::Triangle).to_rgba8();
    Raster::from_rgba(PW, PH, img.into_raw())
}

impl CompState {
    pub fn new() -> Self {
        let mut layers = Vec::new();
        let base = load_preview("assets/mountain_spire.jpg").unwrap_or_else(|| Raster::filled(PW, PH, [40, 60, 110, 255]));
        layers.push(CompLayer { layer: ImageLayer::new("Mountain Spire", base), opacity: KeyTrack::new() });
        if let Some(r) = load_preview("assets/mist_overlay.jpg") {
            let mut l = ImageLayer::new("Mist Overlay", r);
            l.blend = BlendMode::Screen;
            let mut k = KeyTrack::new();
            k.set(0.0, 0.0);
            k.set(2.5, 0.85);
            k.set(5.0, 0.15);
            layers.push(CompLayer { layer: l, opacity: k });
        }
        let mut grade = ImageLayer::new("Gold Grade", Raster::filled(PW, PH, [214, 180, 92, 255]));
        grade.blend = BlendMode::Multiply;
        let mut k = KeyTrack::new();
        k.set(0.0, 0.0);
        k.set(4.0, 0.45);
        layers.push(CompLayer { layer: grade, opacity: k });
        Self { selected: layers.len() - 1, layers, clock: Clock::new(5.0), tex: None, last: None }
    }

    pub fn opacity_at(&self, i: usize, t: f32) -> f32 {
        let l = &self.layers[i];
        l.opacity.sample(t).unwrap_or(l.layer.opacity).clamp(0.0, 1.0)
    }

    pub fn frame(&self, t: f32) -> Raster {
        let ls: Vec<ImageLayer> = self
            .layers
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let mut x = l.layer.clone();
                x.opacity = self.opacity_at(i, t);
                x
            })
            .collect();
        composite(&ls, PW, PH, [0, 0, 0])
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        let t = self.clock.time;
        let ops: Vec<f32> = (0..self.layers.len()).map(|i| self.opacity_at(i, t)).collect();
        let vis: Vec<bool> = self.layers.iter().map(|l| l.layer.visible).collect();
        let blends: Vec<BlendMode> = self.layers.iter().map(|l| l.layer.blend).collect();
        let key = (0.0, ops, vis, blends);
        if self.tex.is_some() && self.last.as_ref() == Some(&key) {
            return;
        }
        let r = self.frame(t);
        let img = egui::ColorImage::from_rgba_unmultiplied([PW as usize, PH as usize], &r.data);
        match &mut self.tex {
            Some(tx) => tx.set(img, egui::TextureOptions::LINEAR),
            None => self.tex = Some(ctx.load_texture("comp", img, egui::TextureOptions::LINEAR)),
        }
        self.last = Some(key);
    }

    pub fn tools(&mut self, ui: &mut egui::Ui, p: &Palette) {
        theme::card(ui, p, "Layers", |ui| {
            for i in (0..self.layers.len()).rev() {
                let on = i == self.selected;
                let l = &mut self.layers[i];
                egui::Frame::none().fill(if on { p.accent_soft() } else { Color32::TRANSPARENT }).rounding(Rounding::same(R_SM + 1.0)).inner_margin(egui::Margin::symmetric(SP_XS, 1.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new(if l.layer.visible { "👁" } else { "–" }).color(p.muted)).frame(false)).clicked() {
                            l.layer.visible = !l.layer.visible;
                        }
                        if ui.add(egui::Button::new(egui::RichText::new(&l.layer.name).color(if on { p.accent } else { p.text })).frame(false)).clicked() {
                            self.selected = i;
                        }
                    });
                });
            }
        });
        theme::card(ui, p, "", |ui| {
            ui.label(theme::muted(p, "Opacity is keyframed per layer and interpolated linearly. Scrub the timeline to preview."));
        });
    }

    pub fn center(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let ctx = ui.ctx().clone();
        self.refresh(&ctx);
        let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
        let rect = resp.rect;
        painter.rect_filled(rect, Rounding::ZERO, p.canvas);
        let zoom = ViewTransform::fit_zoom(rect.width(), rect.height() - 24.0, (PW, PH), 32.0);
        let view = ViewTransform { viewport: (rect.min.x, rect.min.y, rect.width(), rect.height()), image: (PW, PH), zoom, pan: (0.0, 0.0) };
        let (x, y, w, h) = view.image_rect();
        let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
        painter.rect_filled(r.translate(Vec2::new(0.0, 6.0)), Rounding::same(R_MD), Color32::from_black_alpha(if p.dark { 120 } else { 40 }));
        if let Some(t) = &self.tex {
            let mut shape = egui::epaint::RectShape::filled(r, Rounding::same(R_MD), Color32::WHITE);
            shape.fill_texture_id = t.id();
            shape.uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            painter.add(shape);
        }
        painter.rect_stroke(r, Rounding::same(R_MD), Stroke::new(1.0_f32, p.border));
        let tc = format!("PROGRAM  ·  {}:{:02}", format_clock(self.clock.time as u64), ((self.clock.time.fract()) * 30.0) as u32);
        painter.text(r.left_top() + Vec2::new(0.0, -SP_SM), egui::Align2::LEFT_BOTTOM, tc, egui::FontId::monospace(11.5), p.muted);
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let t = self.clock.time;
        let i = self.selected.min(self.layers.len() - 1);
        let cur = self.opacity_at(i, t);
        theme::card(ui, p, "Layer", |ui| {
            let l = &mut self.layers[i];
            ui.label(egui::RichText::new(&l.layer.name).strong().color(p.text));
            let mut pct = cur * 100.0;
            let resp = ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).suffix("%").text("Opacity"));
            if resp.changed() {
                if l.opacity.is_empty() {
                    l.layer.opacity = pct / 100.0;
                } else {
                    l.opacity.set(t, pct / 100.0);
                }
            }
            egui::ComboBox::from_label("Blend").selected_text(l.layer.blend.label()).show_ui(ui, |ui| {
                for m in BlendMode::ALL {
                    ui.selectable_value(&mut l.layer.blend, m, m.label());
                }
            });
            if !l.opacity.is_empty() {
                ui.label(theme::muted(p, "Animated: editing opacity sets a key at the playhead."));
            }
        });
        theme::card(ui, p, "Keyframes", |ui| {
            let l = &mut self.layers[i];
            ui.horizontal(|ui| {
                if ui.add(theme::primary_button(p, "◆ Add key")).clicked() {
                    l.opacity.set(t, cur);
                }
                let near = l.opacity.has_key_near(t, 0.08);
                if ui.add_enabled(near, theme::toggle_button(p, "Delete key", false)).clicked() {
                    l.opacity.remove_near(t, 0.08);
                }
            });
            if l.opacity.is_empty() {
                ui.label(theme::muted(p, "No keys — opacity is constant."));
            }
            let mut jump = None;
            for (kt, kv) in l.opacity.keys() {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new(format!("◆  {:.2}s", kt)).monospace().color(p.text)).frame(false)).on_hover_text("Jump to key").clicked() {
                        jump = Some(*kt);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.0}%", kv * 100.0)).monospace().color(p.muted));
                    });
                });
            }
            if let Some(kt) = jump {
                self.clock.time = kt;
                self.clock.playing = false;
            }
        });
    }

    pub fn timeline(&mut self, ui: &mut egui::Ui, p: &Palette) {
        ui.horizontal(|ui| timeline::transport(ui, p, &mut self.clock));
        ui.add_space(SP_XS);
        let tracks: Vec<Track> = self.layers.iter().enumerate().rev().map(|(i, l)| Track { label: &l.layer.name, keys: &l.opacity, selected: i == self.selected }).collect();
        let n = self.layers.len();
        let out = timeline::ruler(ui, p, "comp-ruler", &mut self.clock, &tracks);
        if let Some(k) = out.picked_track {
            self.selected = n - 1 - k;
        }
        let _ = SP_MD;
    }
}
