//! Rust UI window (egui). Tuxedo theme matching the suite layout.
//!
//! cargo run --features ui --bin ui
//!
//! Top mode tabs, left tool catalog, center canvas,
//! right layers/masks/properties, bottom Maya shelf and status.

use eframe::egui;

fn resolve_asset(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);
    if p.exists() {
        return p.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let next = dir.join(path);
            if next.exists() {
                return next;
            }
            let up = dir.join("..").join(path);
            if up.exists() {
                return up;
            }
        }
    }
    p.to_path_buf()
}
use obs_rust_bitstream::{Joint, JointHierarchy, Keyframes, LayerPixels, LayerStack, LayerStackPixels, LiveWeights, MayaShelf, MayaShelves, Psd, PsGroup, PsToolbar, Reconnect, ToolDispatch, WeightBridge};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_inner_size([1600.0, 1000.0]),
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
    ToonBoom,
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
    blend: String,
    strokes: Vec<(u32, u32, &'static str, String)>,
    time: f32,
    keys: Vec<f32>,
    maya_objects: Vec<(String, f32, f32, f32, bool)>,
    selected_object: String,
    pixel_stack: LayerStackPixels,
    keys_pure: Keyframes,
    joints: JointHierarchy,
    reconnect: Reconnect,
    thumbs: Vec<Option<egui::TextureHandle>>,
    burned: Vec<(String, u8, u8, u8)>,
    onion: bool,
    onion_frames: u32,
    onion_opacity: f32,
    packets: u32,
    brush_size: f32,
    brush_hard: f32,
    brush_space: f32,
    brush_flow: f32,
    filter: String,
    file_path: String,
    recent: Vec<String>,
    weights: LiveWeights,
    dark: bool,
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
            blend: "Normal".to_string(),
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
                s.add(LayerPixels::new("Mist Overlay", "assets/mist_overlay.jpg"));
                s
            },
            keys_pure: Keyframes::new(),
            joints: {
                let mut h = JointHierarchy::new();
                if let Ok(j) = Joint::new("joint1") {
                    h.add(j);
                }
                h
            },
            reconnect: Reconnect::new(),
            thumbs: Vec::new(),
            burned: Vec::new(),
            onion: false,
            onion_frames: 1,
            onion_opacity: 50.0,
            packets: 0,
            brush_size: 10.0,
            brush_hard: 80.0,
            brush_space: 10.0,
            brush_flow: 100.0,
            filter: "None".to_string(),
            file_path: "assets/mountain_spire.psd".to_string(),
            recent: vec![
                "assets/mountain_spire.psd".to_string(),
                "assets/mountain_spire.jpg".to_string(),
            ],
            weights: {
                let mut w = LiveWeights::new();
                w.merge_lessons(&[
                    ("brush.size", 12.0),
                    ("brush.hardness", 0.8),
                    ("filter.levels", 1.2),
                    ("blend.multiply", 0.5),
                ]);
                w
            },
            dark: true,
        }
    }
}

impl eframe::App for TuxedoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg = if self.dark { egui::Color32::from_rgb(10, 10, 10) } else { egui::Color32::from_rgb(245, 245, 245) };
        let fg = if self.dark { egui::Color32::from_rgb(245, 245, 245) } else { egui::Color32::from_rgb(20, 20, 20) };
        let gold = egui::Color32::from_rgb(201, 168, 76);
        let muted = if self.dark { egui::Color32::from_rgb(107, 107, 107) } else { egui::Color32::from_rgb(140, 140, 140) };
        if self.dark {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }


        // Top bar stays in the central area
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("◆ TUXEDO EDIT").color(gold).size(16.0));
                ui.label(egui::RichText::new("Professional Creative Suite").color(muted).size(10.0));
                ui.separator();
                for (m, label) in [
                    (Mode::Photoshop, "PHOTOSHOP"),
                    (Mode::AfterEffects, "AFTER EFFECTS"),
                    (Mode::Maya, "MAYA"),
                    (Mode::Obs, "OBS"),
                    (Mode::ToonBoom, "TOONBOOM"),
                ] {
                    let color = if self.mode == m { gold } else { fg };
                    if ui.button(egui::RichText::new(label).color(color)).clicked() {
                        self.mode = m;
                    }
                }
                ui.separator();
                ui.label(egui::RichText::new("Pro User").color(fg));
                ui.label(egui::RichText::new("LICENSED").color(gold));
                if ui.button(egui::RichText::new(if self.dark { "Dark" } else { "Light" }).color(gold)).clicked() {
                    self.dark = !self.dark;
                }
                ui.separator();
                ui.add(egui::TextEdit::singleline(&mut self.file_path).desired_width(200.0));
                if ui.button(egui::RichText::new("Open").color(fg)).clicked() {
                    if let Some(doc) = Psd::load(std::path::Path::new(&self.file_path)) {
                        self.doc = Some(doc);
                        self.last_action = format!("Opened {}", self.file_path);
                        if !self.recent.contains(&self.file_path) {
                            self.recent.insert(0, self.file_path.clone());
                        }
                    }
                }
                ui.menu_button(egui::RichText::new("Open Recent").color(fg), |ui| {
                    for path in self.recent.clone() {
                        if ui.button(&path).clicked() {
                            self.file_path = path.clone();
                            if path.ends_with(".psd") {
                                if let Some(doc) = Psd::load(std::path::Path::new(&path)) {
                                    self.doc = Some(doc);
                                }
                            }
                            self.last_action = format!("Opened {}", path);
                            ui.close_menu();
                        }
                    }
                });
                if ui.button(egui::RichText::new("Save").color(fg)).clicked() {
                    if let Some(doc) = &self.doc {
                        let _ = doc.save(std::path::Path::new(&self.file_path));
                        self.last_action = format!("Saved {}", self.file_path);
                    }
                }
            });
        });

        // Left tools — reserved width
        egui::SidePanel::left("tools")
            .exact_width(220.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                // Left: tool catalog
                egui::ScrollArea::vertical().id_salt("left").show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(180.0);
                    ui.label(egui::RichText::new("◆ PHOTOSHOP").color(gold));
                    for group in PsToolbar::groups() {
                        ui.label(egui::RichText::new(group.label().to_uppercase()).color(gold));
                        for tool in PsToolbar::in_group(*group) {
                            let text = format!("  {}  {}", tool.name, tool.hotkey);
                            let color = if self.tool == tool.name { gold } else { fg };
                            if ui.button(egui::RichText::new(text).color(color)).clicked() {
                                self.tool = tool.name.to_string();
                            }
                        }
                    }
                });
                });

                ui.separator();


                });
            });

        // Right layers / properties — reserved width
        egui::SidePanel::right("layers")
            .exact_width(280.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                // Right: mode-specific panel
                egui::ScrollArea::vertical().id_salt("right").show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(260.0);
                    match self.mode {
                        Mode::Photoshop => {
                            ui.label(egui::RichText::new("◆ LAYERS").color(gold));
                            let layer_names: Vec<(String, bool)> = self.pixel_stack.layers.iter().map(|l| (l.name.clone(), l.visible)).collect();
                            for (i, (name, visible)) in layer_names.iter().enumerate() {
                                let eye = if *visible { "◉" } else { "○" };
                                let color = if self.selected_layer == *name { gold } else { fg };
                                ui.horizontal(|ui| {
                                    if ui.button(egui::RichText::new(eye).color(color)).clicked() {
                                        self.pixel_stack.toggle(name);
                                        self.last_action = "recomposite".to_string();
                                        self.canvas_texture = None;
                                    }
                                    if let Some(Some(tex)) = self.thumbs.get(i) {
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                                        ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                    }
                                    if ui.button(egui::RichText::new(name).color(color)).clicked() {
                                        self.selected_layer = name.clone();
                                    }
                                });
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("◆ MASKS").color(gold));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Size").color(fg));
                                ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=50.0).suffix(" px"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Space").color(fg));
                                ui.add(egui::Slider::new(&mut self.brush_space, 1.0..=50.0).suffix(" px"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Flow").color(fg));
                                ui.add(egui::Slider::new(&mut self.brush_flow, 10.0..=100.0).suffix("%"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Feather").color(fg));
                                ui.add(egui::Slider::new(&mut self.feather, 0.0..=50.0).suffix(" px"));
                            });
                            ui.separator();
                            ui.label(egui::RichText::new("◆ FILTERS").color(gold));
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("filter")
                                    .selected_text(&self.filter)
                                    .show_ui(ui, |ui| {
                                        for f in ["None", "Levels", "Curves", "Gaussian"] {
                                            if ui.selectable_value(&mut self.filter, f.to_string(), f).clicked() {
                                                self.last_action = "recomposite".to_string();
                                                self.canvas_texture = None;
                                            }
                                        }
                                    });
                            });
                            ui.label(egui::RichText::new(format!("Selected: {}", self.selected_layer)).color(fg));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Blend").color(fg));
                                egui::ComboBox::from_id_salt("blend")
                                    .selected_text(&self.blend)
                                    .show_ui(ui, |ui| {
                                        for mode in ["Normal", "Multiply", "Screen", "Overlay"] {
                                            if ui.selectable_value(&mut self.blend, mode.to_string(), mode).clicked() {
                                                self.last_action = "recomposite".to_string();
                                                self.canvas_texture = None;
                                            }
                                        }
                                    });
                            });
                            if ui.button(egui::RichText::new("Suggest").color(gold)).clicked() {
                                self.weights.merge_lessons(&[
                                    ("brush.size", 16.0),
                                    ("filter.levels", 1.4),
                                ]);
                                let _ = std::fs::write("assets/weights.txt", format!("brush.size={}\nfilter.levels=1.4\n", WeightBridge::read_or(&self.weights, "brush.size", 0.0)));
                                self.last_action = "suggested".to_string();
                            }
                            if ui.button(egui::RichText::new("Save PSD").color(gold)).clicked() {
                                if let Some(doc) = &mut self.doc {
                                    doc.set_opacity(&self.selected_layer, (self.opacity * 2.55) as u8);
                                    let _ = doc.save(std::path::Path::new("assets/mountain_spire.psd"));
                                    self.last_action = format!("Saved burned={}", self.burned.len());
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
                                let _ = self.keys_pure.add("position", (self.time * 1000.0) as u64, self.time);
                            }
                            ui.label(egui::RichText::new(format!("pure keys: {}", self.keys_pure.count("position"))).color(muted));
                            if let Ok(v) = self.keys_pure.sample("position", (self.time * 1000.0) as u64) {
                                ui.label(egui::RichText::new(format!("sampled: {:.2}", v)).color(gold));
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
                                ui.label(egui::RichText::new(format!("pure joints: {}", self.joints.len())).color(muted));
                                if let Ok(j) = self.joints.get(&self.selected_object) {
                                    let (x, y, z) = j.position();
                                    ui.label(egui::RichText::new(format!("pos: {:.1} {:.1} {:.1}", x, y, z)).color(gold));
                                }
                            }
                        }
                        Mode::Obs => {
                            ui.label(egui::RichText::new("◆ STREAM").color(gold));
                            let stream_color = if self.streaming { egui::Color32::from_rgb(0, 200, 0) } else { muted };
                            if ui.button(egui::RichText::new("● Streaming").color(stream_color)).clicked() {
                                self.streaming = !self.streaming;
                                if self.streaming {
                                    self.packets += 1;
                                    let frames = 8u32;
                                    let mut dump = Vec::new();
                                    dump.extend_from_slice(&frames.to_le_bytes());
                                    dump.extend((0..64).map(|i| (i * 3) as u8));
                                    let path = std::path::Path::new("assets/stream_dump.bin");
                                    let _ = std::fs::write(path, &dump);
                                    self.last_action = format!("run {} frames → {}", frames, path.display());
                                }
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
                            ui.label(egui::RichText::new(format!("packets: {}", self.packets)).color(fg));
                            ui.separator();
                            ui.label(egui::RichText::new("◆ RECONNECT").color(gold));
                            ui.label(egui::RichText::new(format!("attempts: {}", self.reconnect.attempts())).color(fg));
                            ui.label(egui::RichText::new(format!("retry: {}s / {}", self.reconnect.retry_sec(), self.reconnect.retry_max())).color(muted));
                            if ui.button(egui::RichText::new("Retry").color(gold)).clicked() {
                                self.reconnect.set_settings(self.reconnect.retry_max() + 1, self.reconnect.retry_sec());
                            }
                        }
                        Mode::ToonBoom => {
                            ui.label(egui::RichText::new("◆ DRAWING").color(gold));
                            let frame = (self.time * 24.0) as u32;
                            ui.label(egui::RichText::new(format!("frame: {}", frame)).color(fg));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Exposure").color(fg));
                                ui.add(egui::Slider::new(&mut self.time, 0.0..=5.0).suffix(" s"));
                            });
                            ui.separator();
                            ui.label(egui::RichText::new("◆ FRAME STRIP").color(gold));
                            ui.horizontal(|ui| {
                                for f in 0..12 {
                                    let t = f as f32 / 24.0;
                                    let near = (self.time - t).abs() < 0.05;
                                    let mut prev = false;
                                    if self.onion {
                                        for n in 1..=self.onion_frames {
                                            if (self.time - t - n as f32 / 24.0).abs() < 0.05 {
                                                prev = true;
                                            }
                                        }
                                    }
                                    let mark = if near { "◆" } else if prev { "◇" } else { "·" };
                                    let color = if near { gold } else if prev { egui::Color32::from_rgb((150.0 * self.onion_opacity / 100.0) as u8, (120.0 * self.onion_opacity / 100.0) as u8, 60) } else { muted };
                                    ui.label(egui::RichText::new(mark).color(color));
                                }
                            });
                            if ui.button(egui::RichText::new(if self.onion { "Onion: On" } else { "Onion: Off" }).color(gold)).clicked() {
                                self.onion = !self.onion;
                            }
                            if self.onion {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Frames").color(fg));
                                    ui.add(egui::Slider::new(&mut self.onion_frames, 1..=2));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Opacity").color(fg));
                                    ui.add(egui::Slider::new(&mut self.onion_opacity, 10.0..=100.0).suffix("%"));
                                });
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("◆ LAYERS").color(gold));
                            for (name, _, _, _, vis) in &self.maya_objects {
                                let eye = if *vis { "◉" } else { "○" };
                                ui.label(egui::RichText::new(format!("{}  {}", eye, name)).color(fg));
                            }
                            ui.separator();
                            if let Ok(v) = self.keys_pure.sample("position", (self.time * 1000.0) as u64) {
                                ui.label(egui::RichText::new(format!("key: {:.2}", v)).color(gold));
                            }
                        }
                    }
                });
                });
            });

        // Center canvas + Maya shelf
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, bg);
                // Center: canvas
                ui.vertical(|ui| {
                    // Canvas with real pixels
                    if self.thumbs.is_empty() {
                        for layer in &self.pixel_stack.layers {
                            if let Ok(img) = image::open(resolve_asset(&layer.path)) {
                                let small = img.resize(32, 32, image::imageops::FilterType::Nearest);
                                let rgba = small.to_rgba8();
                                let size = [rgba.width() as usize, rgba.height() as usize];
                                let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                                self.thumbs.push(Some(ctx.load_texture(&layer.name, color, egui::TextureOptions::LINEAR)));
                            } else {
                                self.thumbs.push(None);
                            }
                        }
                    }
                    if self.canvas_texture.is_none() || self.last_action == "recomposite" {
                        let paths: Vec<_> = self.pixel_stack.visible().iter().map(|l| l.path.clone()).collect();
                        if let Some(first) = paths.first() {
                            if let Ok(base) = image::open(resolve_asset(first)) {
                                let mut rgba = base.to_rgba8();
                                for path in paths.iter().skip(1) {
                                    if let Ok(over) = image::open(resolve_asset(path)) {
                                        let over_rgba = over.to_rgba8();
                                        for (i, p) in rgba.pixels_mut().enumerate() {
                                            if let Some(op) = over_rgba.pixels().nth(i) {
                                                for c in 0..3 {
                                                    let a = p[c] as f32 / 255.0;
                                                    let b = op[c] as f32 / 255.0;
                                                    let mixed = match self.blend.as_str() {
                                                        "Multiply" => a * b,
                                                        "Screen" => 1.0 - (1.0 - a) * (1.0 - b),
                                                        "Overlay" => if a < 0.5 { 2.0 * a * b } else { 1.0 - 2.0 * (1.0 - a) * (1.0 - b) },
                                                        _ => (a + b) / 2.0,
                                                    };
                                                    let feathered = mixed * (1.0 - self.feather / 100.0).max(0.5);
                                                    let scaled = (feathered * self.density / 100.0 * 255.0) as u8;
                                                    p[c] = scaled;
                                                }
                                            }
                                        }
                                    }
                                }
                                let size = [rgba.width() as usize, rgba.height() as usize];
                                if !self.burned.is_empty() {
                                    let count = self.burned.len().min(50);
                                    for (i, p) in rgba.pixels_mut().enumerate().take(count * 20).step_by(20) {
                                        p[0] = 201; p[1] = 168; p[2] = 76;
                                    }
                                }
                                if self.filter == "Levels" {
                                    let lift = WeightBridge::read_or(&self.weights, "filter.levels", 1.2);
                                    for p in rgba.pixels_mut() {
                                        p[0] = ((p[0] as f32 - 10.0).max(0.0) * lift).min(255.0) as u8;
                                        p[1] = ((p[1] as f32 - 10.0).max(0.0) * lift).min(255.0) as u8;
                                        p[2] = ((p[2] as f32 - 10.0).max(0.0) * lift).min(255.0) as u8;
                                    }
                                } else if self.filter == "Curves" {
                                    for p in rgba.pixels_mut() {
                                        for c in 0..3 {
                                            let v = p[c] as f32 / 255.0;
                                            p[c] = (v * v * 255.0) as u8;
                                        }
                                    }
                                } else if self.filter == "Gaussian" {
                                    for p in rgba.pixels_mut() {
                                        let avg = (p[0] as u16 + p[1] as u16 + p[2] as u16) / 3;
                                        p[0] = avg as u8; p[1] = avg as u8; p[2] = avg as u8;
                                    }
                                }
                                let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                                self.canvas_texture = Some(ctx.load_texture("canvas", color, egui::TextureOptions::LINEAR));
                            }
                        }
                        self.last_action.clear();
                    }
                    if let Some(tex) = &self.canvas_texture {
                        let size = tex.size_vec2();
                        let aspect = size.y / size.x;
                        let w = ui.available_width().min(600.0);
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
                        for (sx, sy, kind, layer) in &self.strokes {
                            if layer != &self.selected_layer { continue; }
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
                                    obs_rust_bitstream::CanvasAction::Paint { .. } => {
                                        self.strokes.push((x, y, "paint", self.selected_layer.clone()));
                                        self.burned.push((self.selected_layer.clone(), 201, 168, 76));
                                        let size_w = WeightBridge::read_or(&self.weights, "brush.size", self.brush_size);
                                        let hard_w = WeightBridge::read_or(&self.weights, "brush.hardness", self.brush_hard / 100.0);
                                        let dab = (size_w * hard_w / self.brush_space.max(1.0)) as usize;
                                        for i in 0..dab.max(1) {
                                            let falloff = 1.0 - (i as f32 / dab.max(1) as f32) * (1.0 - self.brush_hard / 100.0);
                                            let flow = self.brush_flow / 100.0;
                                            let v = (201.0 * falloff * flow) as u8;
                                            self.burned.push((self.selected_layer.clone(), v, (168.0 * falloff * flow) as u8, (76.0 * falloff * flow) as u8));
                                        }
                                        self.last_action = "recomposite".to_string();
                                        self.canvas_texture = None;
                                    }
                                    obs_rust_bitstream::CanvasAction::Select { .. } => self.strokes.push((x, y, "select", self.selected_layer.clone())),
                                    obs_rust_bitstream::CanvasAction::Crop { .. } => self.strokes.push((x, y, "crop", self.selected_layer.clone())),
                                    obs_rust_bitstream::CanvasAction::Heal { .. } => self.strokes.push((x, y, "heal", self.selected_layer.clone())),
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("No image — place assets/ next to the binary or run from the repo root").color(muted));
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


            // Gold frame
            let frame = ui.max_rect();
            ui.painter().rect_stroke(frame.shrink(4.0), 0.0, egui::Stroke::new(1.0, gold));

            ui.separator();
            // Status
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Tool: {}  B", self.tool)).color(fg));
                if !self.last_action.is_empty() {
                    ui.label(egui::RichText::new(&self.last_action).color(gold));
                }
                let brush_w = WeightBridge::read_or(&self.weights, "brush.size", 0.0);
                ui.label(egui::RichText::new(format!("w={:.1}", brush_w)).color(muted));
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
