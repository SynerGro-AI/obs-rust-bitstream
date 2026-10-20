//! Seraphina Studio — unified creative-suite shell (egui / eframe).
//!
//! cargo run --release --features ui --bin ui
//!
//! Modes: Stream (real bitstream inspector), Paint (layered raster
//! editor), 3D (layout preview), Animate (frame-by-frame), Composite
//! (keyframed layers). The Seraphina panel talks to an
//! `AssistantBackend`; the shipped one is an offline stub.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod animate;
mod assistant_panel;
mod composite;
mod paint;
mod scene;
mod stream;
mod theme;
mod timeline;

use eframe::egui::{self, Color32, Key, Rounding, Vec2};
use obs_rust_bitstream::assistant::{LearningEvent, WorkspaceContext};
use obs_rust_bitstream::ui_canvas::format_clock;
use theme::{Palette, SP_LG, SP_MD, SP_SM, SP_XS};

/// Find a bundled file: as given, next to the exe, or one/two/three levels up.
pub fn resolve_asset(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);
    if p.is_absolute() || p.exists() {
        return p.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|d| d.to_path_buf());
        for _ in 0..4 {
            if let Some(d) = &dir {
                let c = d.join(path);
                if c.exists() {
                    return c;
                }
                dir = d.parent().map(|x| x.to_path_buf());
            }
        }
    }
    p.to_path_buf()
}

// ---- toasts -------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub kind: ToastKind,
    pub text: String,
    pub born: f64,
}

#[derive(Default)]
pub struct Toasts {
    pub now: f64,
    pub items: Vec<Toast>,
}

impl Toasts {
    const LIFE: f64 = 3.6;

    pub fn push(&mut self, kind: ToastKind, text: &str) {
        self.items.push(Toast { kind, text: text.to_string(), born: self.now });
        if self.items.len() > 4 {
            self.items.remove(0);
        }
    }

    fn show(&mut self, ctx: &egui::Context, p: &Palette) {
        let now = self.now;
        self.items.retain(|t| now - t.born < Self::LIFE);
        if self.items.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -48.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                for t in &self.items {
                    let age = now - t.born;
                    let a = ((age / 0.18).min(1.0) * ((Self::LIFE - age) / 0.35).min(1.0)).clamp(0.0, 1.0) as f32;
                    let dot = match t.kind {
                        ToastKind::Info => p.accent,
                        ToastKind::Success => p.success,
                        ToastKind::Warn => p.warn,
                        ToastKind::Error => p.danger,
                    };
                    egui::Frame::none()
                        .fill(p.surface.gamma_multiply(a))
                        .stroke(egui::Stroke::new(1.0_f32, p.border.gamma_multiply(a)))
                        .rounding(Rounding::same(99.0))
                        .shadow(egui::epaint::Shadow { offset: Vec2::new(0.0, 6.0), blur: 18.0, spread: 0.0, color: Color32::from_black_alpha((60.0 * a) as u8) })
                        .inner_margin(egui::Margin::symmetric(SP_LG, SP_SM))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, dot.gamma_multiply(a));
                                ui.label(egui::RichText::new(&t.text).color(p.text.gamma_multiply(a)).size(13.0));
                            });
                        });
                    ui.add_space(SP_XS);
                }
            });
        ctx.request_repaint();
    }
}

// ---- app ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Stream,
    Paint,
    Scene,
    Animate,
    Composite,
}

impl Mode {
    pub const ALL: [Mode; 5] = [Mode::Stream, Mode::Paint, Mode::Scene, Mode::Animate, Mode::Composite];

    pub fn label(&self) -> &'static str {
        match self {
            Mode::Stream => "Stream",
            Mode::Paint => "Paint",
            Mode::Scene => "3D",
            Mode::Animate => "Animate",
            Mode::Composite => "Composite",
        }
    }

    fn has_timeline(&self) -> bool {
        matches!(self, Mode::Scene | Mode::Animate | Mode::Composite)
    }

    fn index(&self) -> usize {
        Mode::ALL.iter().position(|m| m == self).unwrap_or(0)
    }
}

pub struct App {
    pub dark: bool,
    pub pal: Palette,
    pub mode: Mode,
    pub show_tools: bool,
    pub show_inspector: bool,
    pub show_timeline: bool,
    pub show_assistant: bool,
    pub show_shortcuts: bool,
    pub toasts: Toasts,
    pub stream: stream::StreamState,
    pub paint: paint::PaintState,
    pub scene: scene::SceneState,
    pub anim: animate::AnimState,
    pub comp: composite::CompState,
    pub assistant: assistant_panel::AssistantPanel,
    last_time: Option<f64>,
}

impl App {
    pub fn new(ctx: &egui::Context) -> Self {
        let pal = Palette::dark();
        let mut fonts = egui::FontDefinitions::default();
        // Hack carries geometric glyphs (◆ ● ◉ ↶ ↷ →) the UI font lacks.
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            f.push("Hack".to_owned());
        }
        ctx.set_fonts(fonts);
        theme::apply(ctx, &pal);
        let mut toasts = Toasts::default();
        let paint = paint::PaintState::new(&mut toasts);
        Self {
            dark: true,
            pal,
            mode: Mode::Stream,
            show_tools: true,
            show_inspector: true,
            show_timeline: true,
            show_assistant: true,
            show_shortcuts: false,
            stream: stream::StreamState::new(),
            paint,
            scene: scene::SceneState::new(),
            anim: animate::AnimState::new(),
            comp: composite::CompState::new(),
            assistant: assistant_panel::AssistantPanel::new(),
            toasts,
            last_time: None,
        }
    }

    pub fn set_dark(&mut self, ctx: &egui::Context, dark: bool) {
        self.dark = dark;
        self.pal = Palette::for_mode(dark);
        theme::apply(ctx, &self.pal);
    }

    pub fn current_tool(&self) -> String {
        match self.mode {
            Mode::Stream => format!("Read {} bit{}", self.stream.width, if self.stream.width == 1 { "" } else { "s" }),
            Mode::Paint => self.paint.tool.label().to_string(),
            Mode::Scene => "Orbit".to_string(),
            Mode::Animate => if self.anim.erase { "Eraser" } else { "Pencil" }.to_string(),
            Mode::Composite => "Keyframe".to_string(),
        }
    }

    fn workspace_ctx(&self) -> WorkspaceContext {
        WorkspaceContext { mode: self.mode.label().to_string(), tool: self.current_tool(), detail: String::new() }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let (cmd, shift) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
        let pressed = |k: Key| ctx.input(|i| i.key_pressed(k));
        for (k, m) in [(Key::Num1, Mode::Stream), (Key::Num2, Mode::Paint), (Key::Num3, Mode::Scene), (Key::Num4, Mode::Animate), (Key::Num5, Mode::Composite)] {
            if cmd && pressed(k) {
                self.mode = m;
            }
        }
        if pressed(Key::F1) || pressed(Key::Questionmark) {
            self.show_shortcuts = !self.show_shortcuts;
        }
        if cmd && pressed(Key::Period) {
            self.show_assistant = !self.show_assistant;
        }
        if pressed(Key::Escape) && self.show_shortcuts {
            self.show_shortcuts = false;
        }
        match self.mode {
            Mode::Stream => {
                if pressed(Key::Enter) || pressed(Key::ArrowRight) {
                    self.stream.step(1);
                }
                if pressed(Key::R) && !cmd {
                    self.stream.insp.reset();
                }
            }
            Mode::Paint => {
                if cmd && pressed(Key::Z) {
                    if shift { self.paint.redo() } else { self.paint.undo() }
                } else if cmd && pressed(Key::Y) {
                    self.paint.redo();
                } else if cmd && pressed(Key::Num0) {
                    self.paint.fit();
                } else if cmd && (pressed(Key::Plus) || pressed(Key::Equals)) {
                    self.paint.zoom_by(true);
                } else if cmd && pressed(Key::Minus) {
                    self.paint.zoom_by(false);
                } else if !cmd {
                    for t in paint::Tool::ALL {
                        if pressed(t.key()) && self.paint.tool != t {
                            self.paint.tool = t;
                            self.assistant.observe(LearningEvent::ToolUsed { mode: "Paint".into(), tool: t.label().into() });
                        }
                    }
                    if pressed(Key::Enter) && self.paint.crop.is_some() {
                        self.paint.apply_crop(&mut self.toasts);
                    }
                    if pressed(Key::Escape) {
                        self.paint.crop = None;
                    }
                }
            }
            Mode::Animate => {
                if cmd && pressed(Key::Z) {
                    if shift { self.anim.redo() } else { self.anim.undo() }
                } else if !cmd {
                    if pressed(Key::Space) {
                        self.anim.clock.toggle();
                    }
                    if pressed(Key::Comma) {
                        let c = self.anim.current();
                        self.anim.goto(c.saturating_sub(1));
                    }
                    if pressed(Key::Period) {
                        let c = self.anim.current();
                        self.anim.goto(c + 1);
                    }
                    if pressed(Key::B) {
                        self.anim.erase = false;
                    }
                    if pressed(Key::E) {
                        self.anim.erase = true;
                    }
                }
            }
            Mode::Scene => {
                if pressed(Key::Space) {
                    self.scene.clock.toggle();
                }
                if pressed(Key::Delete) || pressed(Key::Backspace) {
                    self.scene.delete_selected();
                }
            }
            Mode::Composite => {
                if pressed(Key::Space) {
                    self.comp.clock.toggle();
                }
            }
        }
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        let p = self.pal;
        egui::TopBottomPanel::top("top")
            .frame(egui::Frame::none().fill(p.panel).inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).stroke(egui::Stroke::new(1.0_f32, p.border)))
            .show(ctx, |ui| {
                let full = ui.max_rect();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("◆").size(16.0).color(p.accent));
                    ui.label(egui::RichText::new("Seraphina Studio").size(15.0).strong().color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(theme::toggle_button(&p, "?", self.show_shortcuts)).on_hover_text("Keyboard shortcuts  (F1)").clicked() {
                            self.show_shortcuts = !self.show_shortcuts;
                        }
                        let theme_label = if self.dark { "☀  Light" } else { "🌙  Dark" };
                        if ui.add(theme::toggle_button(&p, theme_label, false)).on_hover_text("Switch appearance").clicked() {
                            let d = !self.dark;
                            self.set_dark(ui.ctx(), d);
                        }
                        ui.add_space(SP_SM);
                        if ui.add(theme::toggle_button(&p, "✨ Seraphina", self.show_assistant)).on_hover_text("Assistant panel  (Ctrl+.)").clicked() {
                            self.show_assistant = !self.show_assistant;
                        }
                        if self.mode.has_timeline() && ui.add(theme::toggle_button(&p, "Timeline", self.show_timeline)).clicked() {
                            self.show_timeline = !self.show_timeline;
                        }
                        if ui.add(theme::toggle_button(&p, "Inspector", self.show_inspector)).clicked() {
                            self.show_inspector = !self.show_inspector;
                        }
                        if ui.add(theme::toggle_button(&p, "Tools", self.show_tools)).clicked() {
                            self.show_tools = !self.show_tools;
                        }
                    });
                });
                // centred segmented mode switcher
                let seg_w = 92.0;
                let w = seg_w * Mode::ALL.len() as f32 + 4.0;
                let rect = egui::Rect::from_center_size(egui::pos2(full.center().x, full.center().y), Vec2::new(w, 28.0));
                let labels: Vec<&str> = Mode::ALL.iter().map(|m| m.label()).collect();
                let idx = self.mode.index();
                let picked = ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| theme::segmented(ui, &p, "modes", &labels, idx, seg_w)).inner;
                if let Some(i) = picked {
                    self.mode = Mode::ALL[i];
                }
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        let p = self.pal;
        let now = self.toasts.now;
        egui::TopBottomPanel::bottom("status")
            .exact_height(28.0)
            .frame(egui::Frame::none().fill(p.panel).inner_margin(egui::Margin::symmetric(SP_MD, 4.0)).stroke(egui::Stroke::new(1.0_f32, p.border)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new(self.mode.label()).size(12.0).strong().color(p.text));
                    ui.label(egui::RichText::new("·").color(p.muted));
                    ui.label(egui::RichText::new(self.current_tool()).size(12.0).color(p.muted));
                    let detail = match self.mode {
                        Mode::Stream => {
                            let (b, bit) = self.stream.insp.cursor();
                            format!("{}  ·  byte {}.{} of {}", self.stream.insp.mode().label(), b, bit, self.stream.insp.data().len())
                        }
                        Mode::Paint => {
                            let pos = self.paint.cursor_px.map(|(x, y)| format!("{}, {} px", x, y)).unwrap_or_else(|| "—".into());
                            format!("{} × {}  ·  {:.0}%  ·  {}", self.paint.width, self.paint.height, self.paint.zoom * 100.0, pos)
                        }
                        Mode::Scene => format!("{} objects  ·  {:.2}s", self.scene.objs.len(), self.scene.clock.time),
                        Mode::Animate => format!("frame {} / {}  ·  {} fps", self.anim.current() + 1, self.anim.frames.len(), self.anim.fps),
                        Mode::Composite => format!("{} layers  ·  {:.2}s", self.comp.layers.len(), self.comp.clock.time),
                    };
                    ui.label(egui::RichText::new("·").color(p.muted));
                    ui.label(egui::RichText::new(detail).size(12.0).monospace().color(p.muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (c, t) = if self.assistant.backend.is_connected() { (p.success, "Assistant connected") } else { (p.muted, "Assistant offline") };
                        theme::pill(ui, &p, c, t);
                        if let Some(s) = self.stream.rec_since {
                            theme::pill(ui, &p, p.danger, &format!("REC {}", format_clock((now - s) as u64)));
                        }
                        if let Some(s) = self.stream.live_since {
                            let (c, t) = if self.stream.reconnecting() { (p.warn, "RECONNECTING".to_string()) } else { (p.success, format!("LIVE {}", format_clock((now - s) as u64))) };
                            theme::pill(ui, &p, c, &t).on_hover_text("Simulated — no output is connected");
                        }
                        if self.stream.live_since.is_some() || self.stream.rec_since.is_some() {
                            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
                        }
                    });
                });
            });
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let p = self.pal;
        let mut open = self.show_shortcuts;
        egui::Window::new("Keyboard shortcuts").open(&mut open).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            let rows: &[(&str, &[(&str, &str)])] = &[
                ("Global", &[("Ctrl+1 … 5", "Switch mode"), ("Ctrl+.", "Toggle Seraphina"), ("F1 / ?", "This sheet")]),
                ("Stream", &[("Enter / →", "Step one read"), ("R", "Reset reader")]),
                ("Paint", &[("B E M C I H Z", "Brush, Eraser, Marquee, Crop, Eyedropper, Hand, Zoom"), ("Ctrl+Z / Ctrl+Shift+Z", "Undo / redo"), ("Ctrl+0", "Fit"), ("Ctrl+= / Ctrl+−", "Zoom"), ("Enter / Esc", "Apply / cancel crop"), ("Middle-drag, scroll", "Pan")]),
                ("Animate", &[("Space", "Play / pause"), (", / .", "Previous / next frame"), ("B / E", "Pencil / eraser")]),
                ("3D · Composite", &[("Space", "Play / pause"), ("Delete", "Delete object (3D)")]),
            ];
            for (group, items) in rows {
                ui.label(theme::section(&p, group));
                egui::Grid::new(*group).num_columns(2).spacing([SP_LG, 4.0]).show(ui, |ui| {
                    for (k, v) in *items {
                        ui.label(egui::RichText::new(*k).monospace().color(p.accent));
                        ui.label(egui::RichText::new(*v).color(p.text));
                        ui.end_row();
                    }
                });
                ui.add_space(SP_SM);
            }
        });
        self.show_shortcuts = open;
    }

    /// One frame of UI. Separate from eframe so tests can drive it.
    pub fn frame(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let dt = self.last_time.map(|t| (now - t) as f32).unwrap_or(0.0).clamp(0.0, 0.1);
        self.last_time = Some(now);
        self.toasts.now = now;
        self.stream.tick(now, &mut self.toasts);
        self.scene.tick(dt);
        if self.anim.clock.tick(dt) || self.comp.clock.tick(dt) {
            ctx.request_repaint();
        }
        if self.scene.clock.playing || self.anim.clock.playing || self.comp.clock.playing {
            ctx.request_repaint();
        }
        self.assistant.tick(now);
        self.handle_keys(ctx);

        let p = self.pal;
        self.top_bar(ctx);
        self.status_bar(ctx);

        let side = egui::Frame::none().fill(p.panel).inner_margin(egui::Margin::same(SP_MD)).stroke(egui::Stroke::new(1.0_f32, p.border));
        let wctx = self.workspace_ctx();
        egui::SidePanel::right("assistant").resizable(true).default_width(300.0).width_range(260.0..=480.0).frame(side).show_animated(ctx, self.show_assistant, |ui| {
            self.assistant.ui(ui, &p, now, &wctx);
        });
        egui::SidePanel::left("tools").resizable(true).default_width(210.0).width_range(180.0..=320.0).frame(side).show_animated(ctx, self.show_tools, |ui| {
            ui.spacing_mut().slider_width = (ui.available_width() - 150.0).max(60.0);
            egui::ScrollArea::vertical().id_salt("tools-scroll").show(ui, |ui| match self.mode {
                Mode::Stream => self.stream.tools(ui, &p, now, &mut self.toasts),
                Mode::Paint => {
                    if let Some(t) = self.paint.tools(ui, &p, &mut self.toasts) {
                        self.assistant.observe(LearningEvent::ToolUsed { mode: "Paint".into(), tool: t.label().into() });
                    }
                }
                Mode::Scene => self.scene.tools(ui, &p, &mut self.toasts),
                Mode::Animate => self.anim.tools(ui, &p),
                Mode::Composite => self.comp.tools(ui, &p),
            });
        });
        egui::SidePanel::right("inspector").resizable(true).default_width(280.0).width_range(240.0..=420.0).frame(side).show_animated(ctx, self.show_inspector, |ui| {
            ui.spacing_mut().slider_width = (ui.available_width() - 175.0).max(60.0);
            egui::ScrollArea::vertical().id_salt("inspector-scroll").show(ui, |ui| match self.mode {
                Mode::Stream => self.stream.inspector(ui, &p),
                Mode::Paint => self.paint.inspector(ui, &p, &mut self.toasts),
                Mode::Scene => self.scene.inspector(ui, &p),
                Mode::Animate => self.anim.inspector(ui, &p, &mut self.toasts),
                Mode::Composite => self.comp.inspector(ui, &p),
            });
        });
        let show_tl = self.show_timeline && self.mode.has_timeline();
        egui::TopBottomPanel::bottom("timeline")
            .resizable(false)
            .frame(egui::Frame::none().fill(p.panel).inner_margin(egui::Margin::same(SP_MD)).stroke(egui::Stroke::new(1.0_f32, p.border)))
            .show_animated(ctx, show_tl, |ui| match self.mode {
                Mode::Scene => self.scene.timeline(ui, &p),
                Mode::Animate => self.anim.timeline(ui, &p, &mut self.toasts),
                Mode::Composite => self.comp.timeline(ui, &p),
                _ => {}
            });
        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| match self.mode {
            Mode::Stream => self.stream.center(ui, &p, &mut self.toasts),
            Mode::Paint => self.paint.center(ui, &p, &mut self.toasts),
            Mode::Scene => self.scene.center(ui, &p),
            Mode::Animate => self.anim.center(ui, &p),
            Mode::Composite => self.comp.center(ui, &p),
        });
        self.shortcuts(ctx);
        self.toasts.show(ctx, &p);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame(ctx);
    }
}

fn main() -> eframe::Result<()> {
    let start_mode = std::env::var("SERAPHINA_MODE").ok();
    let start_light = std::env::var("SERAPHINA_LIGHT").is_ok();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Seraphina Studio").with_inner_size([1440.0, 900.0]).with_min_inner_size([1080.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Seraphina Studio",
        options,
        Box::new(move |cc| {
            let mut app = App::new(&cc.egui_ctx);
            if let Some(m) = start_mode.as_deref() {
                if let Some(mode) = Mode::ALL.iter().find(|x| x.label().eq_ignore_ascii_case(m)) {
                    app.mode = *mode;
                }
            }
            if start_light {
                app.set_dark(&cc.egui_ctx, false);
            }
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(t: f64) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1440.0, 900.0))),
            time: Some(t),
            ..Default::default()
        }
    }

    fn run(app: &mut App, ctx: &egui::Context, frames: usize, t0: f64) {
        for i in 0..frames {
            let _ = ctx.run(input(t0 + i as f64 * 0.05), |ctx| app.frame(ctx));
        }
    }

    #[test]
    fn every_mode_renders_dark_and_light() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        for dark in [true, false] {
            app.set_dark(&ctx, dark);
            for m in Mode::ALL {
                app.mode = m;
                run(&mut app, &ctx, 3, 0.0);
            }
        }
    }

    #[test]
    fn panels_can_all_hide() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.show_tools = false;
        app.show_inspector = false;
        app.show_assistant = false;
        app.show_timeline = false;
        app.show_shortcuts = true;
        for m in Mode::ALL {
            app.mode = m;
            run(&mut app, &ctx, 2, 0.0);
        }
    }

    #[test]
    fn stream_step_and_wrap() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.stream.load_sample(&mut app.toasts);
        app.stream.width = 8;
        app.stream.step(3);
        assert_eq!(app.stream.insp.cursor(), (3, 0));
        app.stream.insp.set_mode(obs_rust_bitstream::stream_inspect::CounterMode::Wrap);
        app.stream.step(256);
        assert_eq!(app.stream.insp.cursor().0, 0);
        run(&mut app, &ctx, 2, 0.0);
    }

    #[test]
    fn stream_open_missing_file_is_error_not_panic() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.stream.path = "definitely/missing.bin".into();
        app.stream.open(&mut app.toasts);
        assert!(app.toasts.items.iter().any(|t| t.kind == ToastKind::Error));
    }

    #[test]
    fn paint_stroke_undo_redo() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        let paint_i = app.paint.layers.len() - 1;
        app.paint.begin_stroke();
        app.paint.paint_to((50.0, 50.0));
        app.paint.paint_to((120.0, 60.0));
        assert!(app.paint.layers[paint_i].raster.data.chunks_exact(4).any(|p| p[3] > 0));
        app.paint.undo();
        assert!(app.paint.layers[paint_i].raster.data.chunks_exact(4).all(|p| p[3] == 0));
        app.paint.redo();
        assert!(app.paint.layers[paint_i].raster.data.chunks_exact(4).any(|p| p[3] > 0));
        app.mode = Mode::Paint;
        run(&mut app, &ctx, 2, 0.0);
    }

    #[test]
    fn paint_crop_changes_size() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.paint.crop = Some(obs_rust_bitstream::ui_canvas::PixelRect { x: 10, y: 10, w: 100, h: 50 });
        app.paint.apply_crop(&mut app.toasts);
        assert_eq!((app.paint.width, app.paint.height), (100, 50));
        assert!(app.paint.layers.iter().all(|l| l.raster.width == 100 && l.raster.height == 50));
        app.mode = Mode::Paint;
        run(&mut app, &ctx, 2, 0.0);
    }

    #[test]
    fn animate_frames() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        let n = app.anim.frames.len();
        app.anim.add_frame(&mut app.toasts);
        assert_eq!(app.anim.frames.len(), n + 1);
        app.anim.draw_to((10.0, 10.0));
        app.anim.delete(&mut app.toasts);
        assert_eq!(app.anim.frames.len(), n);
        app.anim.clock.toggle();
        app.mode = Mode::Animate;
        run(&mut app, &ctx, 5, 0.0);
    }

    #[test]
    fn composite_animates_opacity() {
        let app_ctx = egui::Context::default();
        let app = App::new(&app_ctx);
        let last = app.comp.layers.len() - 1;
        assert!(app.comp.opacity_at(last, 0.0) < app.comp.opacity_at(last, 4.0));
        assert_ne!(app.comp.frame(0.0).data, app.comp.frame(4.0).data);
    }

    #[test]
    fn assistant_round_trip_and_learning() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.assistant.input = "how do I use this?".into();
        let w = app.workspace_ctx();
        app.assistant.send(0.0, &w);
        assert!(app.assistant.is_typing());
        app.assistant.tick(5.0);
        assert!(!app.assistant.is_typing());
        assert!(app.assistant.history.last().map(|m| m.text.contains("No model is connected")).unwrap_or(false));
        app.assistant.observe(LearningEvent::Feedback { helpful: true });
        assert_eq!(app.assistant.learner.weights.get("assistant.helpfulness").ok(), Some(0.1));
    }

    #[test]
    fn scene_keys_drive_position() {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        app.scene.clock.time = 1.5;
        app.scene.tick(0.0);
        assert!((app.scene.objs[0].pos[1] - 1.6).abs() < 1e-3);
        app.scene.delete_selected();
        app.mode = Mode::Scene;
        run(&mut app, &ctx, 2, 0.0);
    }

    #[test]
    fn toasts_expire() {
        let mut t = Toasts::default();
        t.push(ToastKind::Info, "hi");
        t.now = 10.0;
        let ctx = egui::Context::default();
        let _ = ctx.run(input(10.0), |ctx| t.show(ctx, &Palette::dark()));
        assert!(t.items.is_empty());
    }
}
