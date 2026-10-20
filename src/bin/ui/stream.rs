//! Stream mode: bitstream inspector over the real `bitstream::Reader`,
//! plus a clearly simulated output (live / record / reconnect) panel.

use crate::theme::{self, Palette, R_SM, SP_MD, SP_SM, SP_XS};
use crate::{resolve_asset, ToastKind, Toasts};
use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use obs_rust_bitstream::stream_inspect::{sample_stream, to_binary, CounterMode, StreamInspector};
use obs_rust_bitstream::Reconnect;

pub const WIDTHS: [u32; 7] = [1, 2, 3, 4, 5, 8, 16];
const RUN_LIMIT: usize = 4096;

pub struct StreamState {
    pub insp: StreamInspector,
    pub source: String,
    pub path: String,
    pub width: u32,
    follow: bool,
    pub live_since: Option<f64>,
    pub rec_since: Option<f64>,
    pub reconnect: Reconnect,
    reconnect_due: Option<f64>,
}

impl StreamState {
    pub fn new() -> Self {
        let default = resolve_asset("out.obsr");
        let (data, source) = match std::fs::read(&default) {
            Ok(d) if !d.is_empty() => (d, "out.obsr".to_string()),
            _ => (sample_stream(), "Built-in sample (320 bytes)".to_string()),
        };
        Self {
            insp: StreamInspector::new(data, CounterMode::Safe),
            source,
            path: "out.h264".to_string(),
            width: 8,
            follow: true,
            live_since: None,
            rec_since: None,
            reconnect: Reconnect::new(),
            reconnect_due: None,
        }
    }

    pub fn open(&mut self, toasts: &mut Toasts) {
        let path = self.path.trim().to_string();
        if path.is_empty() {
            toasts.push(ToastKind::Error, "Enter a file path to open.");
            return;
        }
        match std::fs::read(resolve_asset(&path)) {
            Ok(d) if d.is_empty() => toasts.push(ToastKind::Error, &format!("{} is empty.", path)),
            Ok(d) => {
                let n = d.len();
                self.insp = StreamInspector::new(d, self.insp.mode());
                self.source = path.clone();
                self.follow = true;
                toasts.push(ToastKind::Success, &format!("Loaded {} ({} bytes)", path, n));
            }
            Err(e) => toasts.push(ToastKind::Error, &format!("Couldn't open {}: {}", path, e)),
        }
    }

    pub fn load_sample(&mut self, toasts: &mut Toasts) {
        self.insp = StreamInspector::new(sample_stream(), self.insp.mode());
        self.source = "Built-in sample (320 bytes)".to_string();
        self.follow = true;
        toasts.push(ToastKind::Info, "Loaded the built-in 320-byte sample.");
    }

    pub fn step(&mut self, times: usize) {
        for _ in 0..times {
            self.insp.read(self.width);
        }
        self.follow = true;
    }

    pub fn run(&mut self, toasts: &mut Toasts) {
        let n = self.insp.run(self.width, RUN_LIMIT);
        self.follow = true;
        if self.insp.exhausted() {
            toasts.push(ToastKind::Info, &format!("Reached the end after {} reads.", n));
        } else {
            toasts.push(ToastKind::Info, &format!("Stopped after {} reads (limit per run).", n));
        }
    }

    pub fn tick(&mut self, now: f64, toasts: &mut Toasts) {
        if let Some(due) = self.reconnect_due {
            if now >= due {
                self.reconnect_due = None;
                self.reconnect.success();
                toasts.push(ToastKind::Success, "Simulated reconnect succeeded.");
            }
        }
    }

    pub fn reconnecting(&self) -> bool {
        self.reconnect_due.is_some()
    }

    // ---- panels -------------------------------------------------------

    pub fn tools(&mut self, ui: &mut egui::Ui, p: &Palette, now: f64, toasts: &mut Toasts) {
        theme::card(ui, p, "Source", |ui| {
            ui.label(egui::RichText::new(&self.source).color(p.text).strong());
            ui.label(theme::muted(p, &format!("{} bytes", self.insp.data().len())));
            ui.add_space(SP_XS);
            let resp = ui.add(egui::TextEdit::singleline(&mut self.path).hint_text("path/to/file.h264").desired_width(f32::INFINITY));
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                if ui.add(theme::primary_button(p, "Open")).clicked() || enter {
                    self.open(toasts);
                }
                if ui.add(theme::toggle_button(p, "Sample", false)).on_hover_text("Load a 320-byte sample that crosses byte 255").clicked() {
                    self.load_sample(toasts);
                }
            });
            ui.label(theme::muted(p, "Try out.h264, out.avbn or out.obsr from the repo root."));
        });

        theme::card(ui, p, "Output · simulated", |ui| {
            ui.label(theme::muted(p, "No encoder or RTMP output is connected. These controls only drive the status UI."));
            ui.add_space(SP_XS);
            let live = self.live_since.is_some();
            if ui.add_sized([ui.available_width(), 30.0], theme::toggle_button(p, if live { "■  End stream" } else { "●  Go live" }, live)).clicked() {
                self.live_since = if live { None } else { Some(now) };
                toasts.push(ToastKind::Info, if live { "Stream ended (simulated)." } else { "Live (simulated)." });
            }
            let rec = self.rec_since.is_some();
            if ui.add_sized([ui.available_width(), 30.0], theme::toggle_button(p, if rec { "■  Stop recording" } else { "⏺  Record" }, rec)).clicked() {
                self.rec_since = if rec { None } else { Some(now) };
                toasts.push(ToastKind::Info, if rec { "Recording stopped (simulated)." } else { "Recording (simulated)." });
            }
            ui.add_space(SP_XS);
            let can_drop = live && !self.reconnecting();
            if ui.add_enabled(can_drop, theme::toggle_button(p, "Simulate dropped connection", false)).clicked() {
                if self.reconnect.start() {
                    let wait = self.reconnect.wait_ms() as f64 / 1000.0;
                    self.reconnect_due = Some(now + wait);
                    toasts.push(ToastKind::Warn, &format!("Connection dropped. Retrying in {:.1}s (attempt {}).", wait, self.reconnect.attempts()));
                } else {
                    self.live_since = None;
                    toasts.push(ToastKind::Error, "Reconnect limit reached. Stream ended.");
                }
            }
            theme::kv(ui, p, "Retry delay", format!("{} s", self.reconnect.retry_sec()));
            theme::kv(ui, p, "Max attempts", format!("{}", self.reconnect.retry_max()));
            if let Some(due) = self.reconnect_due {
                theme::kv(ui, p, "Reconnecting in", format!("{:.1} s", (due - now).max(0.0)));
                ui.ctx().request_repaint();
            }
        });
    }

    pub fn center(&mut self, ui: &mut egui::Ui, p: &Palette, toasts: &mut Toasts) {
        // Toolbar
        egui::Frame::none().inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(theme::section(p, "Counter"));
                let idx = if self.insp.mode() == CounterMode::Safe { 0 } else { 1 };
                if let Some(i) = theme::segmented(ui, p, "counter", &["No-wrap · usize", "Wrap · u8"], idx, 120.0) {
                    self.insp.set_mode(if i == 0 { CounterMode::Safe } else { CounterMode::Wrap });
                    self.follow = true;
                }
                ui.add_space(SP_MD);
                ui.label(theme::section(p, "Read"));
                for w in WIDTHS {
                    if ui.add(theme::toggle_button(p, &format!("{}", w), self.width == w)).on_hover_text(format!("Read {} bit{} per step", w, if w == 1 { "" } else { "s" })).clicked() {
                        self.width = w;
                    }
                }
                ui.add_space(SP_MD);
                if ui.add(theme::primary_button(p, "Step")).on_hover_text("Read once  (Enter / →)").clicked() {
                    self.step(1);
                }
                if ui.add(theme::toggle_button(p, "×8", false)).on_hover_text("Read eight times").clicked() {
                    self.step(8);
                }
                if ui.add(theme::toggle_button(p, "Run", false)).on_hover_text("Read until the end (max 4096 reads)").clicked() {
                    self.run(toasts);
                }
                if ui.add(theme::toggle_button(p, "Reset", false)).on_hover_text("Back to byte 0  (R)").clicked() {
                    self.insp.reset();
                    self.follow = true;
                }
            });
        });
        ui.separator();

        let (cur_byte, cur_bit) = self.insp.cursor();
        let data_len = self.insp.data().len();
        let wrap = self.insp.mode() == CounterMode::Wrap;

        // Current byte bit view
        egui::Frame::none().inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::section(p, "Cursor"));
                ui.label(egui::RichText::new(format!("byte {}  ·  bit {}", cur_byte, cur_bit)).monospace().color(p.text));
                ui.add_space(SP_MD);
                match self.insp.data().get(cur_byte).copied() {
                    Some(b) => {
                        for i in 0..8u8 {
                            let bit = (b >> (7 - i)) & 1;
                            let (r, _) = ui.allocate_exact_size(Vec2::new(22.0, 24.0), egui::Sense::hover());
                            let (fill, fg) = if i < cur_bit {
                                (p.surface_hi, p.muted)
                            } else if i == cur_bit {
                                (p.accent, p.accent_text)
                            } else {
                                (p.surface, p.text)
                            };
                            ui.painter().rect(r, Rounding::same(R_SM - 1.0), fill, Stroke::new(1.0_f32, p.border));
                            ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, format!("{}", bit), egui::FontId::monospace(13.0), fg);
                        }
                        ui.label(egui::RichText::new(format!("0x{:02X}", b)).monospace().color(p.muted));
                    }
                    None => {
                        ui.label(egui::RichText::new("End of buffer — further reads return 0").color(p.warn));
                    }
                }
            });
        });
        ui.separator();

        if data_len == 0 {
            theme::empty_state(ui, p, "📄", "No data", "Open a file or load the sample to start stepping.");
            return;
        }

        // Hex grid (virtualised); 16 columns when there is room, else 8.
        let avail_w = ui.available_width();
        let bytes_per_row: usize = if avail_w >= 560.0 { 16 } else { 8 };
        let show_ascii = avail_w >= 120.0 + bytes_per_row as f32 * 34.0;
        let rows = data_len.div_ceil(bytes_per_row);
        let row_h = 22.0;
        let mut area = egui::ScrollArea::vertical().id_salt("hex").auto_shrink([false, false]);
        if self.follow {
            let target = (cur_byte.min(data_len.saturating_sub(1)) / bytes_per_row) as f32 * (row_h + ui.spacing().item_spacing.y);
            let view_h = ui.available_height();
            area = area.vertical_scroll_offset((target - view_h * 0.4).max(0.0));
            self.follow = false;
        }
        let mono = egui::FontId::monospace(12.5);
        let wrapped_once = wrap && self.insp.records().iter().any(|r| r.wrapped);
        area.show_rows(ui, row_h, rows, |ui, range| {
            for row in range {
                let start = row * bytes_per_row;
                ui.horizontal(|ui| {
                    ui.add_space(SP_MD);
                    ui.label(egui::RichText::new(format!("{:08X}", start)).font(mono.clone()).color(p.muted));
                    ui.add_space(SP_SM);
                    for i in 0..bytes_per_row {
                        let idx = start + i;
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(24.0, row_h - 2.0), egui::Sense::hover());
                        if idx >= data_len {
                            continue;
                        }
                        let b = self.insp.data()[idx];
                        let unreachable = wrap && idx > 255;
                        let read = idx < cur_byte || (wrapped_once && idx <= 255);
                        let (fill, fg) = if idx == cur_byte {
                            (p.accent, p.accent_text)
                        } else if unreachable {
                            (Color32::TRANSPARENT, p.muted.gamma_multiply(0.45))
                        } else if read {
                            (p.surface_hi, p.muted)
                        } else {
                            (Color32::TRANSPARENT, p.text)
                        };
                        if fill != Color32::TRANSPARENT {
                            ui.painter().rect_filled(r, Rounding::same(R_SM - 2.0), fill);
                        }
                        ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, format!("{:02X}", b), mono.clone(), fg);
                        if i == 7 && bytes_per_row == 16 {
                            ui.add_space(SP_XS);
                        }
                        resp.on_hover_text(format!(
                            "Byte {} = 0x{:02X} = {}{}",
                            idx,
                            b,
                            to_binary(b as u16, 8),
                            if unreachable { "\nUnreachable: the u8 counter wraps to 0 after byte 255" } else { "" }
                        ));
                    }
                    if !show_ascii {
                        return;
                    }
                    ui.add_space(SP_MD);
                    let ascii: String = (start..(start + bytes_per_row).min(data_len))
                        .map(|i| {
                            let c = self.insp.data()[i];
                            if c.is_ascii_graphic() { c as char } else { '·' }
                        })
                        .collect();
                    ui.label(egui::RichText::new(ascii).font(mono.clone()).color(p.muted));
                });
            }
        });
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let (b, bit) = self.insp.cursor();
        let wraps = self.insp.records().iter().filter(|r| r.wrapped).count();
        let past = self.insp.records().iter().filter(|r| r.past_end).count();
        theme::card(ui, p, "Reader", |ui| {
            theme::kv(ui, p, "Counter", self.insp.mode().label());
            theme::kv(ui, p, "Position", format!("{}.{}", b, bit));
            theme::kv(ui, p, "Bits read", format!("{}", self.insp.total_bits()));
            theme::kv(ui, p, "Reads", format!("{}", self.insp.records().len()));
            theme::kv(ui, p, "Wraps", format!("{}", wraps));
            theme::kv(ui, p, "Zero-padded reads", format!("{}", past));
            ui.add_space(SP_XS);
            let note = match self.insp.mode() {
                CounterMode::Safe => "usize counter: stops at the end and returns 0 bits after it.",
                CounterMode::Wrap => "u8 counter (C ABI): after byte 255 it wraps to byte 0, like the original uint8_t.",
            };
            ui.label(theme::muted(p, note));
        });
        theme::card(ui, p, "Reads", |ui| {
            let recs = self.insp.records();
            if recs.is_empty() {
                ui.label(theme::muted(p, "No reads yet. Press Step."));
                return;
            }
            let row_h = 20.0;
            egui::ScrollArea::vertical().id_salt("reads").max_height(ui.available_height().max(160.0)).auto_shrink([false, true]).show_rows(
                ui,
                row_h,
                recs.len(),
                |ui, range| {
                    for i in range {
                        let r = &recs[recs.len() - 1 - i];
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("#{:<4}", r.index)).monospace().color(p.muted));
                            ui.label(egui::RichText::new(to_binary(r.value, r.width)).monospace().color(p.text));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if r.wrapped {
                                    ui.label(egui::RichText::new("wrap").size(10.5).color(p.warn));
                                }
                                if r.past_end {
                                    ui.label(egui::RichText::new("eof").size(10.5).color(p.muted));
                                }
                                ui.label(egui::RichText::new(format!("{}.{}→{}.{}", r.byte_before, r.bit_before, r.byte_after, r.bit_after)).monospace().size(11.0).color(p.muted));
                                ui.label(egui::RichText::new(format!("0x{:X}", r.value)).monospace().color(p.accent));
                            });
                        });
                    }
                },
            );
        });
    }
}
