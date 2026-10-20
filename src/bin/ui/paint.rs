//! Paint mode: layered raster editor on `ui_canvas`.

use crate::theme::{self, Palette, R_SM, SP_MD, SP_SM, SP_XS};
use crate::{resolve_asset, ToastKind, Toasts};
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use obs_rust_bitstream::ui_canvas::{
    blend_into, composite, save_target, zoom_step, BlendMode, Brush, Filter, History, ImageLayer, PixelRect, Raster, ViewTransform,
};
use obs_rust_bitstream::Psd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Brush,
    Eraser,
    Marquee,
    Crop,
    Eyedropper,
    Hand,
    Zoom,
}

impl Tool {
    pub const ALL: [Tool; 7] = [Tool::Brush, Tool::Eraser, Tool::Marquee, Tool::Crop, Tool::Eyedropper, Tool::Hand, Tool::Zoom];

    pub fn label(&self) -> &'static str {
        match self {
            Tool::Brush => "Brush",
            Tool::Eraser => "Eraser",
            Tool::Marquee => "Marquee",
            Tool::Crop => "Crop",
            Tool::Eyedropper => "Eyedropper",
            Tool::Hand => "Hand",
            Tool::Zoom => "Zoom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Brush => "🖊",
            Tool::Eraser => "◻",
            Tool::Marquee => "⬚",
            Tool::Crop => "✂",
            Tool::Eyedropper => "💧",
            Tool::Hand => "✋",
            Tool::Zoom => "🔍",
        }
    }

    pub fn key(&self) -> egui::Key {
        match self {
            Tool::Brush => egui::Key::B,
            Tool::Eraser => egui::Key::E,
            Tool::Marquee => egui::Key::M,
            Tool::Crop => egui::Key::C,
            Tool::Eyedropper => egui::Key::I,
            Tool::Hand => egui::Key::H,
            Tool::Zoom => egui::Key::Z,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Tool::Brush => "Drag to paint on the Paint layer. A marquee selection clips the brush.",
            Tool::Eraser => "Drag to erase strokes on the Paint layer.",
            Tool::Marquee => "Drag to select a rectangle. Click once to clear it.",
            Tool::Crop => "Drag a crop box, then press Enter or Apply. Esc cancels.",
            Tool::Eyedropper => "Click the image to pick the brush colour.",
            Tool::Hand => "Drag to pan. Middle-drag pans with any tool.",
            Tool::Zoom => "Click to zoom in, Alt-click to zoom out. Ctrl+scroll also zooms.",
        }
    }
}

pub struct PaintState {
    /// Bottom to top. The last layer is always the Paint layer.
    pub layers: Vec<ImageLayer>,
    pub selected: usize,
    pub width: u32,
    pub height: u32,
    pub tool: Tool,
    pub brush: Brush,
    pub filter: Filter,
    pub levels: f32,
    pub zoom: f32,
    pub pan: Vec2,
    fit_pending: bool,
    pub selection: Option<PixelRect>,
    pub crop: Option<PixelRect>,
    drag_from: Option<(f32, f32)>,
    last_dab: Option<(f32, f32)>,
    history: History<Vec<ImageLayer>>,
    base: Option<Raster>,
    base_tex: Option<egui::TextureHandle>,
    paint_tex: Option<egui::TextureHandle>,
    base_dirty: bool,
    paint_dirty: bool,
    pub cursor_px: Option<(u32, u32)>,
    pub path: String,
    pub doc: Option<Psd>,
    pub doc_name: String,
}

fn load_raster(path: &str) -> Result<Raster, String> {
    let img = image::open(resolve_asset(path)).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Raster::from_rgba(w, h, rgba.into_raw()).ok_or_else(|| "bad pixel buffer".to_string())
}

fn placeholder(w: u32, h: u32) -> Raster {
    let mut r = Raster::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let t = y as f32 / h as f32;
            r.data[i] = (40.0 + 60.0 * t) as u8;
            r.data[i + 1] = (50.0 + 70.0 * t) as u8;
            r.data[i + 2] = (90.0 + 80.0 * (1.0 - t)) as u8;
            r.data[i + 3] = 255;
        }
    }
    r
}

impl PaintState {
    pub fn new(toasts: &mut Toasts) -> Self {
        let mut layers = Vec::new();
        match load_raster("assets/mountain_spire.jpg") {
            Ok(r) => layers.push(ImageLayer::new("Mountain Spire", r)),
            Err(_) => {
                toasts.push(ToastKind::Warn, "assets/ not found next to the app — showing a placeholder image.");
                layers.push(ImageLayer::new("Background", placeholder(1168, 784)));
            }
        }
        if let Ok(r) = load_raster("assets/mist_overlay.jpg") {
            let mut l = ImageLayer::new("Mist Overlay", r);
            l.opacity = 0.55;
            l.blend = BlendMode::Screen;
            layers.push(l);
        }
        let (w, h) = (layers[0].raster.width, layers[0].raster.height);
        layers.push(ImageLayer::new("Paint", Raster::new(w, h)));
        let doc = Psd::load(&resolve_asset("assets/mountain_spire.psd"));
        Self {
            selected: layers.len() - 1,
            layers,
            width: w,
            height: h,
            tool: Tool::Brush,
            brush: Brush::default(),
            filter: Filter::None,
            levels: 1.2,
            zoom: 1.0,
            pan: Vec2::ZERO,
            fit_pending: true,
            selection: None,
            crop: None,
            drag_from: None,
            last_dab: None,
            history: History::new(8),
            base: None,
            base_tex: None,
            paint_tex: None,
            base_dirty: true,
            paint_dirty: true,
            cursor_px: None,
            path: "assets/mountain_spire.psd".to_string(),
            doc,
            doc_name: "mountain_spire".to_string(),
        }
    }

    fn paint_index(&self) -> usize {
        self.layers.len() - 1
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    fn checkpoint(&mut self) {
        self.history.push(self.layers.clone());
    }

    fn restore(&mut self, layers: Vec<ImageLayer>) {
        self.layers = layers;
        self.selected = self.selected.min(self.layers.len() - 1);
        let (w, h) = (self.layers[0].raster.width, self.layers[0].raster.height);
        if (w, h) != (self.width, self.height) {
            self.width = w;
            self.height = h;
            self.selection = None;
            self.fit_pending = true;
        }
        self.base_dirty = true;
        self.paint_dirty = true;
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.history.undo(self.layers.clone()) {
            self.restore(prev);
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.history.redo(self.layers.clone()) {
            self.restore(next);
        }
    }

    pub fn fit(&mut self) {
        self.fit_pending = true;
    }

    pub fn zoom_by(&mut self, zoom_in: bool) {
        self.zoom = zoom_step(self.zoom, zoom_in);
    }

    pub fn actual_size(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }

    pub fn apply_crop(&mut self, toasts: &mut Toasts) {
        let Some(rect) = self.crop.and_then(|c| c.clamped(self.width, self.height)) else {
            toasts.push(ToastKind::Error, "Drag a crop box first.");
            return;
        };
        self.checkpoint();
        for l in &mut self.layers {
            let full = l.raster.resized(self.width, self.height);
            if let Some(c) = full.cropped(rect) {
                l.raster = c;
            }
        }
        self.width = rect.w;
        self.height = rect.h;
        self.crop = None;
        self.selection = None;
        self.fit_pending = true;
        self.base_dirty = true;
        self.paint_dirty = true;
        toasts.push(ToastKind::Success, &format!("Cropped to {} × {} px", rect.w, rect.h));
    }

    pub fn clear_paint(&mut self) {
        self.checkpoint();
        let i = self.paint_index();
        self.layers[i].raster = Raster::new(self.width, self.height);
        self.paint_dirty = true;
    }

    pub fn flattened(&self) -> Raster {
        let n = self.paint_index();
        let mut out = composite(&self.layers[..n], self.width, self.height, [0, 0, 0]);
        self.filter.apply(&mut out, self.levels);
        let paint = &self.layers[n];
        if paint.visible {
            blend_into(&mut out, &paint.raster, BlendMode::Normal, paint.opacity);
        }
        out
    }

    pub fn open(&mut self, toasts: &mut Toasts) {
        let path = self.path.trim().to_string();
        let lower = path.to_lowercase();
        if path.is_empty() {
            toasts.push(ToastKind::Error, "Enter a file path to open.");
        } else if lower.ends_with(".psd") {
            match Psd::load(&resolve_asset(&path)) {
                Some(d) => {
                    toasts.push(ToastKind::Info, &format!("Read PSD header: {} × {}, {} layers. Pixel data isn't decoded yet.", d.width, d.height, d.layers.len()));
                    self.doc = Some(d);
                    self.doc_name = std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("document").to_string();
                }
                None => toasts.push(ToastKind::Error, &format!("{} isn't a readable PSD.", path)),
            }
        } else {
            match load_raster(&path) {
                Ok(r) => {
                    self.checkpoint();
                    let name = std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("Image").to_string();
                    let at = self.paint_index();
                    self.layers.insert(at, ImageLayer::new(&name, r));
                    self.selected = at;
                    self.base_dirty = true;
                    toasts.push(ToastKind::Success, &format!("Added layer “{}”", name));
                }
                Err(e) => toasts.push(ToastKind::Error, &format!("Couldn't open {}: {}", path, e)),
            }
        }
    }

    pub fn save_psd(&mut self, toasts: &mut Toasts) {
        let mut d = Psd::new(self.width, self.height);
        for l in &self.layers {
            d.add_layer(&l.name, l.visible, (l.opacity * 255.0).round() as u8);
        }
        let target = save_target(&format!("{}.psd", self.doc_name));
        match d.save(std::path::Path::new(&target)) {
            Ok(()) => toasts.push(ToastKind::Success, &format!("Saved layer structure to {}", target)),
            Err(e) => toasts.push(ToastKind::Error, &format!("Save failed: {}", e)),
        }
    }

    pub fn export_png(&mut self, toasts: &mut Toasts) {
        let flat = self.flattened();
        let target = format!("{}_export.png", self.doc_name);
        match image::save_buffer(&target, &flat.data, flat.width, flat.height, image::ExtendedColorType::Rgba8) {
            Ok(()) => toasts.push(ToastKind::Success, &format!("Exported {} ({} × {})", target, flat.width, flat.height)),
            Err(e) => toasts.push(ToastKind::Error, &format!("Export failed: {}", e)),
        }
    }

    fn refresh_textures(&mut self, ctx: &egui::Context) {
        if self.base_dirty || self.base_tex.is_none() {
            let n = self.paint_index();
            let mut r = composite(&self.layers[..n], self.width, self.height, [0, 0, 0]);
            self.filter.apply(&mut r, self.levels);
            let img = egui::ColorImage::from_rgba_unmultiplied([r.width as usize, r.height as usize], &r.data);
            match &mut self.base_tex {
                Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                None => self.base_tex = Some(ctx.load_texture("paint-base", img, egui::TextureOptions::LINEAR)),
            }
            self.base = Some(r);
            self.base_dirty = false;
        }
        if self.paint_dirty || self.paint_tex.is_none() {
            let i = self.paint_index();
            let r = &self.layers[i].raster;
            let img = egui::ColorImage::from_rgba_unmultiplied([r.width as usize, r.height as usize], &r.data);
            match &mut self.paint_tex {
                Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                None => self.paint_tex = Some(ctx.load_texture("paint-layer", img, egui::TextureOptions::LINEAR)),
            }
            self.paint_dirty = false;
        }
    }

    /// Stamp the brush from the last dab to `pt` (image space).
    pub fn paint_to(&mut self, pt: (f32, f32)) {
        let erase = self.tool == Tool::Eraser;
        let clip = self.selection;
        let i = self.paint_index();
        let brush = self.brush;
        match self.last_dab {
            Some(from) => {
                brush.stroke(&mut self.layers[i].raster, from, pt, erase, clip);
            }
            None => brush.dab(&mut self.layers[i].raster, pt.0, pt.1, erase, clip),
        }
        self.last_dab = Some(pt);
        self.paint_dirty = true;
    }

    pub fn begin_stroke(&mut self) {
        self.checkpoint();
        self.last_dab = None;
    }

    // ---- panels -------------------------------------------------------

    pub fn tools(&mut self, ui: &mut egui::Ui, p: &Palette, toasts: &mut Toasts) -> Option<Tool> {
        let mut picked = None;
        theme::card(ui, p, "File", |ui| {
            let resp = ui.add(egui::TextEdit::singleline(&mut self.path).hint_text("image.jpg / .png or .psd").desired_width(f32::INFINITY));
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.add_sized([ui.available_width(), 28.0], theme::toggle_button(p, "Open", false)).on_hover_text("Images become a new layer; a PSD reads its header and layer list").clicked() || enter {
                self.open(toasts);
            }
            ui.horizontal(|ui| {
                let w = (ui.available_width() - SP_SM) / 2.0;
                if ui.add_sized([w, 28.0], theme::toggle_button(p, "Save PSD", false)).on_hover_text("Writes the layer structure to <name>_edit.psd; never overwrites the source").clicked() {
                    self.save_psd(toasts);
                }
                if ui.add_sized([w, 28.0], theme::primary_button(p, "Export PNG")).clicked() {
                    self.export_png(toasts);
                }
            });
        });
        theme::card(ui, p, "Tools", |ui| {
            for t in Tool::ALL {
                let on = self.tool == t;
                let text = format!("{}   {}", t.icon(), t.label());
                let resp = ui.add_sized([ui.available_width(), 30.0], theme::toggle_button(p, &text, on).wrap_mode(egui::TextWrapMode::Truncate));
                let resp = resp.on_hover_text(format!("{}  ({})", t.hint(), t.key().name()));
                if resp.clicked() && !on {
                    self.tool = t;
                    picked = Some(t);
                }
            }
        });
        theme::card(ui, p, "How to use", |ui| {
            ui.label(theme::muted(p, self.tool.hint()));
        });
        picked
    }

    pub fn center(&mut self, ui: &mut egui::Ui, p: &Palette, toasts: &mut Toasts) {
        let ctx = ui.ctx().clone();
        self.refresh_textures(&ctx);

        // toolbar
        egui::Frame::none().inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}  ·  {} × {}", self.doc_name, self.width, self.height)).color(p.muted).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::toggle_button(p, "+", false)).on_hover_text("Zoom in  (Ctrl+=)").clicked() {
                        self.zoom_by(true);
                    }
                    ui.label(egui::RichText::new(format!("{:.0}%", self.zoom * 100.0)).monospace().color(p.muted));
                    if ui.add(theme::toggle_button(p, "−", false)).on_hover_text("Zoom out  (Ctrl+−)").clicked() {
                        self.zoom_by(false);
                    }
                    if ui.add(theme::toggle_button(p, "1:1", false)).on_hover_text("Actual pixels").clicked() {
                        self.actual_size();
                    }
                    if ui.add(theme::toggle_button(p, "Fit", false)).on_hover_text("Fit to window  (Ctrl+0)").clicked() {
                        self.fit();
                    }
                    ui.add_space(SP_SM);
                    if ui.add_enabled(self.can_redo(), theme::toggle_button(p, "↷", false)).on_hover_text("Redo  (Ctrl+Shift+Z)").clicked() {
                        self.redo();
                    }
                    if ui.add_enabled(self.can_undo(), theme::toggle_button(p, "↶", false)).on_hover_text("Undo  (Ctrl+Z)").clicked() {
                        self.undo();
                    }
                });
            });
        });

        let avail = ui.available_size();
        let (resp, painter) = ui.allocate_painter(avail, egui::Sense::click_and_drag());
        let rect = resp.rect;
        painter.rect_filled(rect, Rounding::ZERO, p.canvas);
        if self.fit_pending && rect.width() > 10.0 {
            self.zoom = ViewTransform::fit_zoom(rect.width(), rect.height(), (self.width, self.height), 32.0);
            self.pan = Vec2::ZERO;
            self.fit_pending = false;
        }
        let mut view = ViewTransform { viewport: (rect.min.x, rect.min.y, rect.width(), rect.height()), image: (self.width, self.height), zoom: self.zoom, pan: (self.pan.x, self.pan.y) };

        // zoom/pan input
        if resp.hovered() {
            let (scroll, zoom_delta, cmd) = ctx.input(|i| (i.raw_scroll_delta, i.zoom_delta(), i.modifiers.command));
            let pointer = resp.hover_pos().unwrap_or(rect.center());
            let mut new_zoom = self.zoom;
            if (zoom_delta - 1.0).abs() > 1e-3 {
                new_zoom = (self.zoom * zoom_delta).clamp(0.05, 16.0);
            } else if cmd && scroll.y.abs() > 0.0 {
                new_zoom = (self.zoom * (1.0 + scroll.y * 0.0015)).clamp(0.05, 16.0);
            } else if scroll != Vec2::ZERO {
                self.pan += scroll;
            }
            if new_zoom != self.zoom {
                let before = view.to_image(pointer.x, pointer.y);
                self.zoom = new_zoom;
                view.zoom = new_zoom;
                let after = view.to_screen(before.0, before.1);
                self.pan += Vec2::new(pointer.x - after.0, pointer.y - after.1);
            }
            view.zoom = self.zoom;
            view.pan = (self.pan.x, self.pan.y);
        }
        let panning = resp.dragged_by(egui::PointerButton::Middle) || (self.tool == Tool::Hand && resp.dragged_by(egui::PointerButton::Primary));
        if panning {
            self.pan += resp.drag_delta();
            view.pan = (self.pan.x, self.pan.y);
        }

        let (ix, iy, iw, ih) = view.image_rect();
        let img_rect = Rect::from_min_size(Pos2::new(ix, iy), Vec2::new(iw, ih));
        let clip = painter.with_clip_rect(rect);
        // drop shadow + image
        clip.rect_filled(img_rect.translate(Vec2::new(0.0, 6.0)).expand(2.0), Rounding::same(4.0), Color32::from_black_alpha(if p.dark { 120 } else { 40 }));
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        if let Some(t) = &self.base_tex {
            clip.image(t.id(), img_rect, uv, Color32::WHITE);
        }
        let paint = &self.layers[self.paint_index()];
        if paint.visible {
            if let Some(t) = &self.paint_tex {
                clip.image(t.id(), img_rect, uv, Color32::from_white_alpha((paint.opacity * 255.0) as u8));
            }
        }

        // cursor tracking
        let hover = resp.hover_pos();
        self.cursor_px = hover.and_then(|h| {
            let (x, y) = view.to_image(h.x, h.y);
            if view.in_image(x, y) { Some((x as u32, y as u32)) } else { None }
        });

        // tool interaction
        let primary = !panning && (resp.dragged_by(egui::PointerButton::Primary) || resp.clicked());
        let pointer_img = resp.interact_pointer_pos().map(|pp| view.to_image(pp.x, pp.y));
        match self.tool {
            Tool::Brush | Tool::Eraser => {
                if resp.drag_started_by(egui::PointerButton::Primary) || (resp.clicked() && self.drag_from.is_none()) {
                    self.begin_stroke();
                }
                if primary {
                    if let Some(pt) = pointer_img {
                        self.paint_to(pt);
                    }
                }
                if resp.drag_stopped() || resp.clicked() {
                    self.last_dab = None;
                }
            }
            Tool::Marquee | Tool::Crop => {
                if resp.drag_started_by(egui::PointerButton::Primary) {
                    self.drag_from = pointer_img;
                }
                if let (Some(a), Some(b), true) = (self.drag_from, pointer_img, resp.dragged_by(egui::PointerButton::Primary)) {
                    let r = PixelRect::from_corners(a, b).clamped(self.width, self.height);
                    if self.tool == Tool::Marquee {
                        self.selection = r;
                    } else {
                        self.crop = r;
                    }
                }
                if resp.drag_stopped() {
                    self.drag_from = None;
                }
                if resp.clicked() {
                    if self.tool == Tool::Marquee {
                        self.selection = None;
                    } else {
                        self.crop = None;
                    }
                }
            }
            Tool::Eyedropper => {
                if resp.clicked() {
                    if let (Some((x, y)), Some(base)) = (pointer_img, &self.base) {
                        if let Some(px) = base.pixel(x.max(0.0) as u32, y.max(0.0) as u32) {
                            self.brush.color = [px[0], px[1], px[2]];
                            toasts.push(ToastKind::Info, &format!("Picked #{:02X}{:02X}{:02X}", px[0], px[1], px[2]));
                        }
                    }
                }
            }
            Tool::Zoom => {
                if resp.clicked() {
                    let out = ctx.input(|i| i.modifiers.alt);
                    let pointer = resp.interact_pointer_pos().unwrap_or(rect.center());
                    let before = view.to_image(pointer.x, pointer.y);
                    self.zoom = zoom_step(self.zoom, !out);
                    view.zoom = self.zoom;
                    let after = view.to_screen(before.0, before.1);
                    self.pan += Vec2::new(pointer.x - after.0, pointer.y - after.1);
                }
            }
            Tool::Hand => {}
        }

        // overlays
        let to_screen_rect = |r: PixelRect| {
            let a = view.to_screen(r.x as f32, r.y as f32);
            let b = view.to_screen((r.x + r.w) as f32, (r.y + r.h) as f32);
            Rect::from_min_max(Pos2::new(a.0, a.1), Pos2::new(b.0, b.1))
        };
        if let Some(sel) = self.selection {
            let r = to_screen_rect(sel);
            let t = ctx.input(|i| i.time) as f32;
            let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
            clip.add(egui::Shape::dashed_line_with_offset(&pts, Stroke::new(1.5_f32, Color32::WHITE), &[6.0], &[4.0], (t * 20.0) % 10.0));
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
        if let Some(c) = self.crop {
            let r = to_screen_rect(c);
            let shade = Color32::from_black_alpha(140);
            for s in [
                Rect::from_min_max(img_rect.min, Pos2::new(img_rect.max.x, r.min.y)),
                Rect::from_min_max(Pos2::new(img_rect.min.x, r.max.y), img_rect.max),
                Rect::from_min_max(Pos2::new(img_rect.min.x, r.min.y), Pos2::new(r.min.x, r.max.y)),
                Rect::from_min_max(Pos2::new(r.max.x, r.min.y), Pos2::new(img_rect.max.x, r.max.y)),
            ] {
                clip.rect_filled(s, Rounding::ZERO, shade);
            }
            clip.rect_stroke(r, Rounding::ZERO, Stroke::new(1.5_f32, p.accent));
            let label = format!("{} × {}  ·  Enter to crop, Esc to cancel", c.w, c.h);
            let pos = Pos2::new(r.center().x, r.max.y + 16.0);
            let galley = clip.layout_no_wrap(label, egui::FontId::proportional(12.0), Color32::WHITE);
            let bg = Rect::from_center_size(pos, galley.size() + Vec2::new(16.0, 8.0));
            clip.rect_filled(bg, Rounding::same(99.0), Color32::from_black_alpha(200));
            clip.galley(bg.center() - galley.size() / 2.0, galley, Color32::WHITE);
        }
        if let (Some(h), Tool::Brush | Tool::Eraser) = (hover, self.tool) {
            let r = (self.brush.size / 2.0 * self.zoom).max(1.0);
            clip.circle_stroke(h, r, Stroke::new(1.0_f32, Color32::from_white_alpha(200)));
            clip.circle_stroke(h, r + 1.0, Stroke::new(1.0_f32, Color32::from_black_alpha(160)));
        }
        let cursor = match self.tool {
            Tool::Hand => if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab },
            Tool::Zoom => egui::CursorIcon::ZoomIn,
            Tool::Brush | Tool::Eraser => egui::CursorIcon::None,
            _ => egui::CursorIcon::Crosshair,
        };
        if resp.hovered() {
            ctx.set_cursor_icon(if panning { egui::CursorIcon::Grabbing } else { cursor });
        }
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui, p: &Palette, toasts: &mut Toasts) {
        theme::card(ui, p, "Document", |ui| {
            theme::kv(ui, p, "Canvas", format!("{} × {} px", self.width, self.height));
            theme::kv(ui, p, "Layers", format!("{}", self.layers.len()));
            match &self.doc {
                Some(d) => {
                    theme::kv(ui, p, "PSD header", format!("{} × {}", d.width, d.height));
                    theme::kv(ui, p, "PSD layers", format!("{}", d.layers.len()));
                }
                None => {
                    ui.label(theme::muted(p, "No PSD loaded."));
                }
            }
        });

        theme::card(ui, p, "Layers", |ui| {
            let n = self.layers.len();
            let mut toggle = None;
            let mut select = None;
            let mut move_up = None;
            for i in (0..n).rev() {
                let l = &self.layers[i];
                let on = i == self.selected;
                let fill = if on { p.accent_soft() } else { Color32::TRANSPARENT };
                egui::Frame::none().fill(fill).rounding(Rounding::same(R_SM + 1.0)).inner_margin(egui::Margin::symmetric(SP_XS, 2.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let eye = if l.visible { "👁" } else { "–" };
                        if ui.add(egui::Button::new(egui::RichText::new(eye).color(if l.visible { p.text } else { p.muted })).frame(false)).on_hover_text(if l.visible { "Hide layer" } else { "Show layer" }).clicked() {
                            toggle = Some(i);
                        }
                        let name = egui::RichText::new(&l.name).color(if on { p.accent } else { p.text });
                        if ui.add(egui::Button::new(name).frame(false)).clicked() {
                            select = Some(i);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if i + 1 < n - 1 && i < n - 1 {
                                if ui.add(egui::Button::new(egui::RichText::new("⏶").color(p.muted)).frame(false)).on_hover_text("Move up").clicked() {
                                    move_up = Some(i);
                                }
                            }
                            ui.label(egui::RichText::new(format!("{:.0}%", l.opacity * 100.0)).size(11.0).color(p.muted));
                        });
                    });
                });
            }
            if let Some(i) = toggle {
                self.layers[i].visible = !self.layers[i].visible;
                if i == self.paint_index() { self.paint_dirty = true } else { self.base_dirty = true }
            }
            if let Some(i) = select {
                self.selected = i;
            }
            if let Some(i) = move_up {
                self.layers.swap(i, i + 1);
                if self.selected == i { self.selected = i + 1 } else if self.selected == i + 1 { self.selected = i }
                self.base_dirty = true;
            }
        });

        let sel = self.selected.min(self.layers.len() - 1);
        let is_paint = sel == self.paint_index();
        theme::card(ui, p, "Layer", |ui| {
            let l = &mut self.layers[sel];
            ui.label(egui::RichText::new(&l.name).color(p.text).strong());
            let mut pct = l.opacity * 100.0;
            if ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).suffix("%").text("Opacity")).changed() {
                l.opacity = pct / 100.0;
                if is_paint { self.paint_dirty = true } else { self.base_dirty = true }
            }
            ui.add_enabled_ui(!is_paint, |ui| {
                egui::ComboBox::from_label("Blend").selected_text(l.blend.label()).show_ui(ui, |ui| {
                    for m in BlendMode::ALL {
                        if ui.selectable_value(&mut l.blend, m, m.label()).changed() {
                            self.base_dirty = true;
                        }
                    }
                });
            });
            if is_paint {
                ui.label(theme::muted(p, "Brush and eraser always draw here."));
                if ui.add(theme::toggle_button(p, "Clear paint layer", false)).clicked() {
                    self.clear_paint();
                }
            }
        });

        theme::card(ui, p, "Brush", |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Colour").color(p.muted).size(12.0));
                egui::color_picker::color_edit_button_srgb(ui, &mut self.brush.color);
                ui.label(egui::RichText::new(format!("#{:02X}{:02X}{:02X}", self.brush.color[0], self.brush.color[1], self.brush.color[2])).monospace().color(p.muted));
            });
            ui.add(egui::Slider::new(&mut self.brush.size, 1.0..=200.0).logarithmic(true).suffix(" px").text("Size"));
            let mut hard = self.brush.hardness * 100.0;
            if ui.add(egui::Slider::new(&mut hard, 0.0..=100.0).suffix("%").text("Hardness")).changed() {
                self.brush.hardness = hard / 100.0;
            }
            let mut flow = self.brush.flow * 100.0;
            if ui.add(egui::Slider::new(&mut flow, 1.0..=100.0).suffix("%").text("Flow")).changed() {
                self.brush.flow = flow / 100.0;
            }
            let mut sp = self.brush.spacing * 100.0;
            if ui.add(egui::Slider::new(&mut sp, 5.0..=100.0).suffix("%").text("Spacing")).changed() {
                self.brush.spacing = sp / 100.0;
            }
        });

        theme::card(ui, p, "Adjustment", |ui| {
            egui::ComboBox::from_label("Filter").selected_text(self.filter.label()).show_ui(ui, |ui| {
                for f in Filter::ALL {
                    if ui.selectable_value(&mut self.filter, f, f.label()).changed() {
                        self.base_dirty = true;
                    }
                }
            });
            if self.filter == Filter::Levels && ui.add(egui::Slider::new(&mut self.levels, 0.5..=2.0).text("Gain")).changed() {
                self.base_dirty = true;
            }
            ui.label(theme::muted(p, "Applies to image layers below Paint."));
        });

        theme::card(ui, p, "Selection", |ui| {
            match self.selection {
                Some(s) => theme::kv(ui, p, "Marquee", format!("{}×{} at {},{}", s.w, s.h, s.x, s.y)),
                None => {
                    ui.label(theme::muted(p, "None — the brush paints everywhere."));
                }
            }
            ui.horizontal(|ui| {
                if ui.add(theme::toggle_button(p, "Select all", false)).clicked() {
                    self.selection = Some(PixelRect { x: 0, y: 0, w: self.width, h: self.height });
                }
                if ui.add_enabled(self.selection.is_some(), theme::toggle_button(p, "Deselect", false)).clicked() {
                    self.selection = None;
                }
            });
            if self.crop.is_some() {
                ui.add_space(SP_XS);
                ui.horizontal(|ui| {
                    if ui.add(theme::primary_button(p, "Apply crop")).clicked() {
                        self.apply_crop(toasts);
                    }
                    if ui.add(theme::toggle_button(p, "Cancel", false)).clicked() {
                        self.crop = None;
                    }
                });
            }
        });
    }
}
