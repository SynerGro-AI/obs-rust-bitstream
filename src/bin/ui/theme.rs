//! Design system: palette, type scale, spacing and shared widgets.

use eframe::egui::{self, Color32, FontId, Rounding, Stroke, TextStyle, Vec2};

pub const SP_XS: f32 = 4.0;
pub const SP_SM: f32 = 8.0;
pub const SP_MD: f32 = 12.0;
pub const SP_LG: f32 = 16.0;
pub const R_SM: f32 = 6.0;
pub const R_MD: f32 = 10.0;
pub const R_LG: f32 = 14.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub surface_hi: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub success: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub canvas: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            dark: true,
            bg: Color32::from_rgb(14, 14, 16),
            panel: Color32::from_rgb(22, 22, 25),
            surface: Color32::from_rgb(30, 30, 34),
            surface_hi: Color32::from_rgb(42, 42, 47),
            border: Color32::from_rgb(52, 52, 58),
            text: Color32::from_rgb(236, 236, 240),
            muted: Color32::from_rgb(142, 142, 150),
            accent: Color32::from_rgb(214, 180, 92),
            accent_text: Color32::from_rgb(20, 18, 12),
            success: Color32::from_rgb(52, 199, 89),
            danger: Color32::from_rgb(255, 69, 58),
            warn: Color32::from_rgb(255, 159, 10),
            canvas: Color32::from_rgb(10, 10, 12),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            bg: Color32::from_rgb(242, 242, 247),
            panel: Color32::from_rgb(249, 249, 251),
            surface: Color32::from_rgb(255, 255, 255),
            surface_hi: Color32::from_rgb(234, 234, 239),
            border: Color32::from_rgb(216, 216, 222),
            text: Color32::from_rgb(28, 28, 30),
            muted: Color32::from_rgb(110, 110, 118),
            accent: Color32::from_rgb(168, 128, 32),
            accent_text: Color32::from_rgb(255, 255, 255),
            success: Color32::from_rgb(36, 160, 70),
            danger: Color32::from_rgb(215, 40, 35),
            warn: Color32::from_rgb(200, 120, 0),
            canvas: Color32::from_rgb(222, 222, 228),
        }
    }

    pub fn for_mode(dark: bool) -> Self {
        if dark {
            Self::dark()
        } else {
            Self::light()
        }
    }

    /// Opaque accent tint for selection fills. Mixed on the CPU so it
    /// looks the same on every GPU backend (alpha blending in sRGB vs
    /// linear space varies between drivers).
    pub fn accent_soft(&self) -> Color32 {
        mix(self.surface, self.accent, if self.dark { 0.16 } else { 0.14 })
    }
}

/// Linear mix of two opaque colours.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Apply palette, type scale and spacing to the context.
pub fn apply(ctx: &egui::Context, p: &Palette) {
    let mut style = (*ctx.style()).clone();
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.panel;
    v.window_fill = p.surface;
    v.extreme_bg_color = p.bg;
    v.faint_bg_color = p.surface_hi;
    v.code_bg_color = p.surface_hi;
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent_soft();
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);
    v.window_rounding = Rounding::same(R_LG);
    v.menu_rounding = Rounding::same(R_MD);
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.slider_trailing_fill = true;
    v.striped = false;
    let w = &mut v.widgets;
    for (state, fill) in [
        (&mut w.noninteractive, p.surface),
        (&mut w.inactive, p.surface_hi),
        (&mut w.hovered, p.border),
        (&mut w.active, p.border),
        (&mut w.open, p.surface_hi),
    ] {
        state.rounding = Rounding::same(R_SM + 2.0);
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.fg_stroke = Stroke::new(1.0_f32, p.text);
        state.expansion = 0.0;
    }
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text);
    w.inactive.bg_stroke = Stroke::NONE;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, p.accent.gamma_multiply(0.6));
    w.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    style.visuals = v;

    style.spacing.item_spacing = Vec2::new(SP_SM, 6.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.interact_size.y = 26.0;
    style.spacing.slider_width = 150.0;
    style.spacing.window_margin = egui::Margin::same(SP_LG);
    style.spacing.menu_margin = egui::Margin::same(SP_SM);
    style.spacing.scroll = egui::style::ScrollStyle::thin();
    style.animation_time = 0.16;
    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(17.0)),
        (TextStyle::Body, FontId::proportional(13.5)),
        (TextStyle::Button, FontId::proportional(13.5)),
        (TextStyle::Small, FontId::proportional(11.0)),
        (TextStyle::Monospace, FontId::monospace(12.5)),
    ]
    .into();
    ctx.set_style(style);
}

/// Rounded card with an optional small-caps title.
pub fn card<R>(ui: &mut egui::Ui, p: &Palette, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(R_MD))
        .inner_margin(egui::Margin::same(SP_MD))
        .outer_margin(egui::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: SP_SM })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if !title.is_empty() {
                ui.label(section(p, title));
                ui.add_space(SP_XS);
            }
            add(ui)
        })
        .inner
}

/// Small-caps section label.
pub fn section(p: &Palette, text: &str) -> egui::RichText {
    egui::RichText::new(text.to_uppercase()).size(10.5).color(p.muted).strong()
}

pub fn muted(p: &Palette, text: &str) -> egui::RichText {
    egui::RichText::new(text).color(p.muted).size(12.0)
}

/// Filled accent button.
pub fn primary_button(p: &Palette, text: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text.to_string()).color(p.accent_text).strong())
        .fill(p.accent)
        .rounding(Rounding::same(R_SM + 2.0))
}

/// Quiet button that reads as selected when `on`.
pub fn toggle_button(p: &Palette, text: &str, on: bool) -> egui::Button<'static> {
    let (fill, color) = if on { (p.accent_soft(), p.accent) } else { (p.surface_hi, p.text) };
    egui::Button::new(egui::RichText::new(text.to_string()).color(color))
        .fill(fill)
        .stroke(if on { Stroke::new(1.0_f32, p.accent.gamma_multiply(0.7)) } else { Stroke::NONE })
        .rounding(Rounding::same(R_SM + 2.0))
}

/// Status pill: coloured dot plus text.
pub fn pill(ui: &mut egui::Ui, p: &Palette, color: Color32, text: &str) -> egui::Response {
    egui::Frame::none()
        .fill(p.surface_hi)
        .rounding(Rounding::same(99.0))
        .inner_margin(egui::Margin::symmetric(SP_SM, 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 3.5, color);
                ui.label(egui::RichText::new(text).size(11.5).color(p.text));
            });
        })
        .response
}

/// iOS-style segmented control with an animated thumb.
/// Returns the new index when the user picks a segment.
pub fn segmented(ui: &mut egui::Ui, p: &Palette, id: &str, labels: &[&str], selected: usize, seg_w: f32) -> Option<usize> {
    let n = labels.len().max(1);
    let h = 28.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(seg_w * n as f32 + 4.0, h), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, Rounding::same(R_SM + 3.0), p.surface_hi);
    let t = ui.ctx().animate_value_with_time(egui::Id::new(id).with("thumb"), selected as f32, 0.18);
    let thumb = egui::Rect::from_min_size(rect.min + Vec2::new(2.0 + t * seg_w, 2.0), Vec2::new(seg_w, h - 4.0));
    painter.rect(thumb, Rounding::same(R_SM + 1.0), if p.dark { p.border } else { p.surface }, Stroke::new(1.0_f32, p.border));
    let mut picked = None;
    for (i, label) in labels.iter().enumerate() {
        let r = egui::Rect::from_min_size(rect.min + Vec2::new(2.0 + i as f32 * seg_w, 2.0), Vec2::new(seg_w, h - 4.0));
        let resp = ui.interact(r, egui::Id::new(id).with(i), egui::Sense::click());
        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        if resp.clicked() && i != selected {
            picked = Some(i);
        }
        let color = if i == selected { p.text } else if resp.hovered() { p.text.gamma_multiply(0.85) } else { p.muted };
        painter.text(r.center(), egui::Align2::CENTER_CENTER, *label, FontId::proportional(12.5), color);
    }
    picked
}

/// Key-value row used in inspectors.
pub fn kv(ui: &mut egui::Ui, p: &Palette, key: &str, value: impl Into<String>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(key).color(p.muted).size(12.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value.into()).color(p.text).size(12.5).monospace());
        });
    });
}

/// Empty-state block centred in the available space.
pub fn empty_state(ui: &mut egui::Ui, p: &Palette, icon: &str, title: &str, body: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.3).max(SP_LG));
        ui.label(egui::RichText::new(icon).size(34.0).color(p.muted));
        ui.add_space(SP_SM);
        ui.label(egui::RichText::new(title).size(16.0).color(p.text).strong());
        ui.add_space(SP_XS);
        ui.label(egui::RichText::new(body).size(12.5).color(p.muted));
    });
}
