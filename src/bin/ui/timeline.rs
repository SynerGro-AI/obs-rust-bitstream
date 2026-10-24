//! Shared transport + keyframe ruler.

use crate::commands::AppCommand;
use crate::theme::{self, Palette, R_SM, SP_SM};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};
use egui_phosphor::regular as ic;
use obs_rust_bitstream::ui_canvas::{format_clock, KeyTrack};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Clock {
    pub time: f32,
    pub duration: f32,
    pub playing: bool,
    pub looping: bool,
}

impl Clock {
    pub fn new(duration: f32) -> Self {
        Self {
            time: 0.0,
            duration: duration.max(0.1),
            playing: false,
            looping: true,
        }
    }

    /// Advance by `dt` seconds while playing. Returns true if time moved.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.playing {
            return false;
        }
        self.time += dt.max(0.0);
        if self.time > self.duration {
            if self.looping {
                self.time %= self.duration;
            } else {
                self.time = self.duration;
                self.playing = false;
            }
        }
        true
    }

    pub fn toggle(&mut self) {
        if !self.playing && self.time >= self.duration {
            self.time = 0.0;
        }
        self.playing = !self.playing;
    }
}

/// Apply transport commands to a clock. `None` = not a transport command.
pub fn apply(clock: &mut Clock, cmd: &AppCommand) -> Option<Result<String, String>> {
    match cmd {
        AppCommand::TimelinePlay { playing } => {
            let want = playing.unwrap_or(!clock.playing);
            if want != clock.playing {
                clock.toggle();
            }
            Some(Ok(if clock.playing {
                "playing".into()
            } else {
                format!("paused at {:.2}s", clock.time)
            }))
        }
        AppCommand::TimelineScrub { time } => {
            if !time.is_finite() {
                return Some(Err("time must be a number".into()));
            }
            clock.time = time.clamp(0.0, clock.duration);
            Some(Ok(format!("playhead at {:.2}s", clock.time)))
        }
        _ => None,
    }
}

/// Play/pause, jump to start/end, loop toggle and the time readout.
pub fn transport(ui: &mut egui::Ui, p: &Palette, clock: &mut Clock, bus: &mut Vec<AppCommand>) {
    if theme::icon_button(ui, p, ic::SKIP_BACK, "Go to start", false).clicked() {
        bus.push(AppCommand::TimelineScrub { time: 0.0 });
    }
    let label = if clock.playing {
        format!("{}  Pause", ic::PAUSE)
    } else {
        format!("{}  Play", ic::PLAY)
    };
    if ui
        .add(theme::primary_button(p, &label))
        .on_hover_text("Play / pause  (Space)")
        .clicked()
    {
        bus.push(AppCommand::TimelinePlay {
            playing: Some(!clock.playing),
        });
    }
    if theme::icon_button(ui, p, ic::SKIP_FORWARD, "Go to end", false).clicked() {
        bus.push(AppCommand::TimelinePlay {
            playing: Some(false),
        });
        bus.push(AppCommand::TimelineScrub {
            time: clock.duration,
        });
    }
    if theme::icon_button(ui, p, ic::REPEAT, "Loop playback", clock.looping).clicked() {
        clock.looping = !clock.looping;
    }
    let frac = clock.time.fract();
    ui.label(
        egui::RichText::new(format!(
            "{}.{:02}",
            format_clock(clock.time as u64),
            (frac * 100.0) as u32
        ))
        .monospace()
        .color(p.text),
    );
    ui.label(
        egui::RichText::new(format!("/ {:.1}s", clock.duration))
            .monospace()
            .color(p.muted),
    );
}

pub struct Track<'a> {
    pub label: &'a str,
    pub keys: &'a KeyTrack,
    pub selected: bool,
}

pub struct RulerOut {
    /// Track row clicked (label column).
    pub picked_track: Option<usize>,
}

/// Time ruler with per-track keyframe diamonds. Dragging scrubs.
pub fn ruler(
    ui: &mut egui::Ui,
    p: &Palette,
    id: &str,
    clock: &mut Clock,
    tracks: &[Track<'_>],
) -> RulerOut {
    let label_w = 140.0;
    let ruler_h = 22.0;
    let row_h = 24.0;
    let h = ruler_h + row_h * tracks.len().max(1) as f32 + 4.0;
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), h), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let lane = Rect::from_min_max(
        Pos2::new(rect.min.x + label_w, rect.min.y),
        Pos2::new(rect.max.x - SP_SM, rect.max.y),
    );
    let x_of = |t: f32| lane.min.x + (t / clock.duration).clamp(0.0, 1.0) * lane.width();

    painter.rect_filled(lane, CornerRadius::same(R_SM as u8), p.surface);
    // ticks
    let step = if clock.duration > 20.0 {
        5.0
    } else if clock.duration > 6.0 {
        1.0
    } else {
        0.5
    };
    let mut t = 0.0;
    while t <= clock.duration + 1e-3 {
        let x = x_of(t);
        let major = (t / (step * 2.0)).fract().abs() < 1e-3;
        painter.line_segment(
            [
                Pos2::new(x, lane.min.y + if major { 4.0 } else { 12.0 }),
                Pos2::new(x, lane.min.y + ruler_h),
            ],
            Stroke::new(1.0_f32, p.border),
        );
        if major {
            painter.text(
                Pos2::new(x + 3.0, lane.min.y + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{:.0}s", t),
                egui::FontId::monospace(10.0),
                p.muted,
            );
        }
        t += step;
    }
    let mut out = RulerOut { picked_track: None };
    for (i, tr) in tracks.iter().enumerate() {
        let y = rect.min.y + ruler_h + i as f32 * row_h;
        let row = Rect::from_min_max(
            Pos2::new(rect.min.x, y),
            Pos2::new(rect.max.x - SP_SM, y + row_h),
        );
        if tr.selected {
            painter.rect_filled(row, CornerRadius::same(R_SM as u8), p.accent_soft());
        }
        let lab = Rect::from_min_max(row.min, Pos2::new(lane.min.x, row.max.y));
        let resp = ui.interact(
            lab,
            egui::Id::new(id).with("track").with(i),
            egui::Sense::click(),
        );
        if resp.clicked() {
            out.picked_track = Some(i);
        }
        painter.text(
            Pos2::new(row.min.x + SP_SM, row.center().y),
            egui::Align2::LEFT_CENTER,
            tr.label,
            egui::FontId::proportional(12.0),
            if tr.selected { p.accent } else { p.text },
        );
        painter.line_segment(
            [
                Pos2::new(lane.min.x, row.max.y),
                Pos2::new(lane.max.x, row.max.y),
            ],
            Stroke::new(1.0_f32, p.border.gamma_multiply(0.5)),
        );
        let keys = tr.keys.keys();
        for w in keys.windows(2) {
            painter.line_segment(
                [
                    Pos2::new(x_of(w[0].0), row.center().y),
                    Pos2::new(x_of(w[1].0), row.center().y),
                ],
                Stroke::new(2.0_f32, p.accent.gamma_multiply(0.35)),
            );
        }
        for (kt, _) in keys {
            let c = Pos2::new(x_of(*kt), row.center().y);
            let s = 5.0;
            let pts = vec![
                c + Vec2::new(0.0, -s),
                c + Vec2::new(s, 0.0),
                c + Vec2::new(0.0, s),
                c + Vec2::new(-s, 0.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                pts,
                p.accent,
                Stroke::new(1.0_f32, p.bg),
            ));
        }
    }
    // scrub
    let scrub = ui.interact(
        lane,
        egui::Id::new(id).with("scrub"),
        egui::Sense::click_and_drag(),
    );
    if let Some(pos) = scrub.interact_pointer_pos() {
        if scrub.dragged() || scrub.clicked() {
            clock.time = ((pos.x - lane.min.x) / lane.width()).clamp(0.0, 1.0) * clock.duration;
            clock.playing = false;
        }
    }
    let scrub = scrub.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
    let _ = scrub;
    let x = x_of(clock.time);
    painter.line_segment(
        [Pos2::new(x, lane.min.y), Pos2::new(x, lane.max.y)],
        Stroke::new(1.5_f32, p.accent),
    );
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(x - 6.0, lane.min.y),
            Pos2::new(x + 6.0, lane.min.y),
            Pos2::new(x, lane.min.y + 8.0),
        ],
        p.accent,
        Stroke::NONE,
    ));
    let _ = Color32::TRANSPARENT;
    out
}
