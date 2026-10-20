//! Seraphina side panel: chat over an `AssistantBackend`, feedback
//! routed to an `OnlineLearner` for small incremental weight updates.

use crate::theme::{self, Palette, R_LG, SP_MD, SP_SM, SP_XS};
use eframe::egui::{self, Rounding, Stroke};
use obs_rust_bitstream::assistant::{AssistantBackend, ChatMessage, LearningEvent, OfflineBackend, OnlineLearner, Role, WeightNudgeLearner, WorkspaceContext};

pub struct AssistantPanel {
    pub backend: Box<dyn AssistantBackend>,
    pub learner: WeightNudgeLearner,
    pub history: Vec<ChatMessage>,
    rated: Vec<Option<bool>>,
    pub input: String,
    pending: Option<(String, f64)>,
    pub last_update: Option<(String, f32)>,
}

impl AssistantPanel {
    pub fn new() -> Self {
        let mut s = Self {
            backend: Box::new(OfflineBackend),
            learner: WeightNudgeLearner::new(0.1),
            history: Vec::new(),
            rated: Vec::new(),
            input: String::new(),
            pending: None,
            last_update: None,
        };
        s.push(ChatMessage::new(
            Role::Assistant,
            "Hi, I'm Seraphina's panel. No model is connected yet, so I can only offer built-in tips for the current mode. Ask anything to see how the chat flow works.",
        ));
        s
    }

    fn push(&mut self, m: ChatMessage) {
        self.history.push(m);
        self.rated.push(None);
    }

    pub fn is_typing(&self) -> bool {
        self.pending.is_some()
    }

    pub fn send(&mut self, now: f64, ctx: &WorkspaceContext) {
        let text = self.input.trim().to_string();
        if text.is_empty() || self.pending.is_some() {
            return;
        }
        self.input.clear();
        self.push(ChatMessage::new(Role::User, &text));
        let reply = match self.backend.respond(&self.history, ctx) {
            Ok(r) => r,
            Err(e) => format!("Couldn't answer: {:?}", e),
        };
        self.pending = Some((reply, now + 0.6));
    }

    pub fn tick(&mut self, now: f64) {
        if let Some((_, due)) = &self.pending {
            if now >= *due {
                if let Some((text, _)) = self.pending.take() {
                    self.push(ChatMessage::new(Role::Assistant, &text));
                }
            }
        }
    }

    pub fn observe(&mut self, e: LearningEvent) {
        if let Some(u) = self.learner.observe(&e) {
            self.last_update = Some(u);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, p: &Palette, now: f64, wctx: &WorkspaceContext) {
        ui.add_space(SP_SM);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("✨ Seraphina").size(15.0).strong().color(p.accent));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(egui::RichText::new("Clear").size(11.5).color(p.muted)).frame(false)).clicked() {
                    self.history.truncate(1);
                    self.rated.truncate(1);
                    self.pending = None;
                }
                let (c, t) = if self.backend.is_connected() { (p.success, "Connected") } else { (p.warn, "Offline stub") };
                theme::pill(ui, p, c, t).on_hover_text(self.backend.name().to_string());
            });
        });
        ui.label(theme::muted(p, &format!("Context: {} · {}", wctx.mode, wctx.tool)));
        ui.add_space(SP_XS);

        // input pinned at the bottom
        let mut send = false;
        egui::TopBottomPanel::bottom("assistant-input").frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, SP_SM))).show_inside(ui, |ui| {
            egui::CollapsingHeader::new(theme::section(p, &format!("Online learning · {} updates", self.learner.updates()))).id_salt("learning").show(ui, |ui| {
                ui.label(theme::muted(p, "Feedback and tool use nudge LiveWeights by ±0.1 per event (clamped). Nothing is trained or sent anywhere."));
                let mut names: Vec<&str> = self.learner.weights.names();
                names.sort();
                if names.is_empty() {
                    ui.label(theme::muted(p, "No weights yet."));
                }
                for n in names.iter().take(12) {
                    theme::kv(ui, p, n, format!("{:.2}", self.learner.weights.get(n).unwrap_or(0.0)));
                }
            });
            ui.add_space(SP_XS);
            ui.horizontal(|ui| {
                let w = ui.available_width() - 64.0;
                let r = ui.add_sized([w.max(80.0), 30.0], egui::TextEdit::singleline(&mut self.input).hint_text("Ask Seraphina…").margin(egui::Margin::symmetric(SP_SM, 6.0)));
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    send = true;
                    r.request_focus();
                }
                let can = !self.input.trim().is_empty() && self.pending.is_none();
                if ui.add_enabled(can, theme::primary_button(p, "Send")).clicked() {
                    send = true;
                }
            });
        });
        if send {
            self.send(now, wctx);
        }

        let mut feedback = None;
        egui::ScrollArea::vertical().id_salt("chat").stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            let max_w = ui.available_width() * 0.88;
            for (i, m) in self.history.iter().enumerate() {
                let user = m.role == Role::User;
                let layout = egui::Layout::top_down(if user { egui::Align::Max } else { egui::Align::Min });
                ui.with_layout(layout, |ui| {
                    let (fill, fg) = if user { (p.accent, p.accent_text) } else { (p.surface_hi, p.text) };
                    let mut rounding = Rounding::same(R_LG);
                    if user { rounding.se = 4.0 } else { rounding.sw = 4.0 }
                    egui::Frame::none().fill(fill).rounding(rounding).inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).show(ui, |ui| {
                        ui.set_max_width(max_w);
                        ui.add(egui::Label::new(egui::RichText::new(&m.text).color(fg).size(13.0)).wrap());
                    });
                });
                if m.role == Role::Assistant && i > 0 {
                    ui.horizontal(|ui| {
                        ui.add_space(SP_XS);
                        let r = self.rated[i];
                        let up = egui::RichText::new("👍").size(15.0).color(if r == Some(true) { p.accent } else { p.muted });
                        let down = egui::RichText::new("👎").size(15.0).color(if r == Some(false) { p.accent } else { p.muted });
                        if ui.add_enabled(r.is_none(), egui::Button::new(up).frame(false)).on_hover_text("Helpful").clicked() {
                            feedback = Some((i, true));
                        }
                        if ui.add_enabled(r.is_none(), egui::Button::new(down).frame(false)).on_hover_text("Not helpful").clicked() {
                            feedback = Some((i, false));
                        }
                    });
                }
                ui.add_space(SP_XS);
            }
            if self.pending.is_some() {
                egui::Frame::none().fill(p.surface_hi).rounding(Rounding::same(R_LG)).inner_margin(egui::Margin::symmetric(SP_MD, SP_SM)).stroke(Stroke::NONE).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for k in 0..3 {
                            let a = (((now * 6.0) as f32 - k as f32 * 0.8).sin() * 0.5 + 0.5) * 0.7 + 0.3;
                            let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(r.center(), 3.0, p.muted.gamma_multiply(a));
                        }
                    });
                });
                ui.ctx().request_repaint();
            }
        });
        if let Some((i, helpful)) = feedback {
            self.rated[i] = Some(helpful);
            self.observe(LearningEvent::Feedback { helpful });
        }
    }
}
