//! 3D mode: wireframe viewport with an outliner, Select/Move/Rotate/
//! Scale gizmo drags, editable cube meshes (extrude, bevel) and a
//! keyframed Y channel. Wireframe only: no renderer behind it yet.

use crate::commands::AppCommand;
use crate::theme::{self, Palette, R_SM, SP_MD, SP_SM, SP_XS};
use crate::timeline::{self, Clock, Track};
use crate::{ToastKind, Toasts};
use eframe::egui::{self, Color32, Pos2, Rounding, Stroke, Vec2};
use obs_rust_bitstream::ui_canvas::KeyTrack;
use obs_rust_bitstream::{Joint, JointHierarchy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Cube,
    Joint,
    Light,
    Camera,
}

impl Kind {
    fn icon(&self) -> &'static str {
        match self {
            Kind::Cube => "⬛",
            Kind::Joint => "◉",
            Kind::Light => "☀",
            Kind::Camera => "🎥",
        }
    }
}

/// Edge mesh in object space. `top` is the current top face ring.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub verts: Vec<[f32; 3]>,
    pub edges: Vec<(usize, usize)>,
    pub top: Vec<usize>,
    pub faces: usize,
}

impl Mesh {
    pub fn cube() -> Self {
        let h = 0.5;
        let verts = vec![[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h], [-h, h, -h], [h, h, -h], [h, h, h], [-h, h, h]];
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];
        Self { verts, edges, top: vec![4, 5, 6, 7], faces: 6 }
    }

    fn add_ring(&mut self, ring: Vec<[f32; 3]>, side_faces: usize) {
        let base = self.verts.len();
        let n = ring.len();
        self.verts.extend(ring);
        for k in 0..n {
            self.edges.push((base + k, base + (k + 1) % n));
            self.edges.push((self.top[k], base + k));
        }
        self.top = (base..base + n).collect();
        self.faces += side_faces;
    }

    /// Extrude the top face straight up by `d`.
    pub fn extrude(&mut self, d: f32) {
        let ring: Vec<[f32; 3]> = self.top.iter().map(|&i| {
            let v = self.verts[i];
            [v[0], v[1] + d, v[2]]
        }).collect();
        let n = ring.len();
        self.add_ring(ring, n);
    }

    /// Chamfer the top face: inset toward its centre by `a` and lift.
    pub fn bevel(&mut self, a: f32) {
        let a = a.clamp(0.02, 0.45);
        let n = self.top.len() as f32;
        let c = self.top.iter().fold([0.0f32; 3], |acc, &i| [acc[0] + self.verts[i][0] / n, acc[1] + self.verts[i][1] / n, acc[2] + self.verts[i][2] / n]);
        let ring: Vec<[f32; 3]> = self.top.iter().map(|&i| {
            let v = self.verts[i];
            [v[0] + (c[0] - v[0]) * a, v[1] + a * 0.5, v[2] + (c[2] - v[2]) * a]
        }).collect();
        let k = ring.len();
        self.add_ring(ring, k);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gizmo {
    Select,
    Move,
    Rotate,
    Scale,
}

impl Gizmo {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "select" => Some(Gizmo::Select),
            "move" | "translate" => Some(Gizmo::Move),
            "rotate" => Some(Gizmo::Rotate),
            "scale" => Some(Gizmo::Scale),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Gizmo::Select => "Select",
            Gizmo::Move => "Move",
            Gizmo::Rotate => "Rotate",
            Gizmo::Scale => "Scale",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub name: String,
    pub kind: Kind,
    pub pos: [f32; 3],
    pub rot_y: f32,
    pub scale: f32,
    pub visible: bool,
    pub parent: Option<usize>,
    pub track_y: KeyTrack,
    pub mesh: Option<Mesh>,
}

impl Obj {
    fn new(name: &str, kind: Kind, pos: [f32; 3]) -> Self {
        let mesh = (kind == Kind::Cube).then(Mesh::cube);
        Self { name: name.to_string(), kind, pos, rot_y: 0.0, scale: 1.0, visible: true, parent: None, track_y: KeyTrack::new(), mesh }
    }
}

pub struct SceneState {
    pub objs: Vec<Obj>,
    pub selected: Option<usize>,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub grid: bool,
    pub clock: Clock,
    pub joints: JointHierarchy,
    pub gizmo: Gizmo,
    pub shelf_tab: usize,
    counter: usize,
}

impl SceneState {
    pub fn new() -> Self {
        let mut objs = vec![
            Obj::new("cube1", Kind::Cube, [0.0, 0.5, 0.0]),
            Obj::new("hip", Kind::Joint, [2.2, 1.0, 0.0]),
            Obj::new("knee", Kind::Joint, [2.4, 0.5, 0.3]),
            Obj::new("key_light", Kind::Light, [-2.5, 3.0, 2.0]),
            Obj::new("camera1", Kind::Camera, [0.0, 1.5, 5.0]),
        ];
        objs[2].parent = Some(1);
        objs[0].track_y.set(0.0, 0.5);
        objs[0].track_y.set(1.5, 1.6);
        objs[0].track_y.set(3.0, 0.5);
        let mut joints = JointHierarchy::new();
        for o in objs.iter().filter(|o| o.kind == Kind::Joint) {
            if let Ok(j) = Joint::new(&o.name) {
                joints.add(j);
            }
        }
        let _ = joints.set_parent("knee", "hip");
        Self { objs, selected: Some(0), yaw: 0.6, pitch: 0.45, dist: 11.0, grid: true, clock: Clock::new(3.0), joints, gizmo: Gizmo::Select, shelf_tab: 0, counter: 1 }
    }

    pub fn tick(&mut self, dt: f32) {
        self.clock.tick(dt);
        let t = self.clock.time;
        for o in &mut self.objs {
            if let Some(y) = o.track_y.sample(t) {
                o.pos[1] = y;
            }
        }
        for o in self.objs.iter().filter(|o| o.kind == Kind::Joint) {
            if let Ok(j) = self.joints.get_mut(&o.name) {
                j.set_position(o.pos[0], o.pos[1], o.pos[2]);
            }
        }
    }

    pub fn add(&mut self, kind: Kind, toasts: &mut Toasts) {
        self.counter += 1;
        let base = match kind {
            Kind::Cube => "cube",
            Kind::Joint => "joint",
            Kind::Light => "light",
            Kind::Camera => "camera",
        };
        let name = format!("{}{}", base, self.counter);
        if kind == Kind::Joint {
            if let Ok(j) = Joint::new(&name) {
                self.joints.add(j);
            }
        }
        self.objs.push(Obj::new(&name, kind, [0.0, 0.5, 0.0]));
        self.selected = Some(self.objs.len() - 1);
        toasts.push(ToastKind::Success, &format!("Added {}", name));
    }

    pub fn delete_selected(&mut self) {
        let Some(i) = self.selected else { return };
        if i >= self.objs.len() {
            return;
        }
        self.objs.remove(i);
        for o in &mut self.objs {
            o.parent = match o.parent {
                Some(p) if p == i => None,
                Some(p) if p > i => Some(p - 1),
                other => other,
            };
        }
        self.selected = if self.objs.is_empty() { None } else { Some(i.min(self.objs.len() - 1)) };
    }

    pub fn find(&self, name: Option<&str>) -> Result<usize, String> {
        match name.map(|n| n.trim()).filter(|n| !n.is_empty()) {
            None => self.selected.filter(|i| *i < self.objs.len()).ok_or_else(|| "no object selected".to_string()),
            Some(n) => self.objs.iter().position(|o| o.name.eq_ignore_ascii_case(n)).ok_or_else(|| format!("no object named '{}' (objects: {})", n, self.objs.iter().map(|o| o.name.as_str()).collect::<Vec<_>>().join(", "))),
        }
    }

    /// Execute a 3D command. `None` = not a 3D command.
    pub fn apply(&mut self, cmd: &AppCommand, toasts: &mut Toasts) -> Option<Result<String, String>> {
        use AppCommand as C;
        Some(match cmd {
            C::SceneAdd { kind } => {
                let k = match kind.to_lowercase().as_str() {
                    "cube" => Kind::Cube,
                    "joint" => Kind::Joint,
                    "light" => Kind::Light,
                    "camera" => Kind::Camera,
                    other => return Some(Err(format!("unknown kind '{}'", other))),
                };
                self.add(k, toasts);
                Ok(format!("added {}", self.objs.last().map(|o| o.name.clone()).unwrap_or_default()))
            }
            C::SceneSelect { object } => self.find(Some(object)).map(|i| {
                self.selected = Some(i);
                format!("selected {}", self.objs[i].name)
            }),
            C::SceneGizmo { mode } => match Gizmo::from_str(mode) {
                Some(g) => {
                    self.gizmo = g;
                    Ok(format!("{} tool", g.label()))
                }
                None => Err(format!("unknown gizmo '{}'", mode)),
            },
            C::SceneTransform { object, translate, rotate_y, scale } => self.find(object.as_deref()).map(|i| {
                let o = &mut self.objs[i];
                let mut parts = Vec::new();
                if let Some(t) = translate {
                    for k in 0..3 {
                        o.pos[k] += t[k];
                    }
                    parts.push(format!("moved {:.2}, {:.2}, {:.2}", t[0], t[1], t[2]));
                }
                if let Some(r) = rotate_y {
                    o.rot_y = *r;
                    parts.push(format!("rotated Y {:.0}°", r));
                }
                if let Some(s) = scale {
                    o.scale = s.clamp(0.05, 20.0);
                    parts.push(format!("scaled {:.2}", o.scale));
                }
                format!("{} {}", o.name, parts.join(", "))
            }),
            C::SceneExtrude { distance } => self.find(None).and_then(|i| {
                let o = &mut self.objs[i];
                let d = distance.unwrap_or(0.5).clamp(-5.0, 5.0);
                let m = o.mesh.as_mut().ok_or_else(|| format!("{} has no mesh to extrude", o.name))?;
                m.extrude(d);
                Ok(format!("extruded {} top face by {:.2}", o.name, d))
            }),
            C::SceneBevel { amount } => self.find(None).and_then(|i| {
                let o = &mut self.objs[i];
                let a = amount.unwrap_or(0.15);
                let m = o.mesh.as_mut().ok_or_else(|| format!("{} has no mesh to bevel", o.name))?;
                m.bevel(a);
                Ok(format!("bevelled {} top face ({:.2})", o.name, a.clamp(0.02, 0.45)))
            }),
            C::SetKeyframe => self.find(None).map(|i| {
                let t = self.clock.time;
                let o = &mut self.objs[i];
                o.track_y.set(t, o.pos[1]);
                format!("keyed {} Y={:.2} at {:.2}s", o.name, o.pos[1], t)
            }),
            C::DeleteKeyframe => self.find(None).and_then(|i| {
                let t = self.clock.time;
                let o = &mut self.objs[i];
                if o.track_y.remove_near(t, 0.05) { Ok(format!("deleted {} key at {:.2}s", o.name, t)) } else { Err(format!("no key on {} near {:.2}s", o.name, t)) }
            }),
            C::TimelinePlay { .. } | C::TimelineScrub { .. } => return timeline::apply(&mut self.clock, cmd),
            C::SceneDelete { object } => self.find(object.as_deref()).map(|i| {
                self.selected = Some(i);
                let n = self.objs[i].name.clone();
                self.delete_selected();
                format!("deleted {}", n)
            }),
            _ => return None,
        })
    }

    pub fn context(&self) -> String {
        let objs: Vec<String> = self.objs.iter().enumerate().map(|(i, o)| format!("{}{} ({:?} at {:.1},{:.1},{:.1} rotY {:.0} scale {:.2}{})", if self.selected == Some(i) { "*" } else { "" }, o.name, o.kind, o.pos[0], o.pos[1], o.pos[2], o.rot_y, o.scale, o.mesh.as_ref().map(|m| format!(", {} faces", m.faces)).unwrap_or_default())).collect();
        format!("3D: gizmo {}, objects (* = selected): {}", self.gizmo.label(), objs.join("; "))
    }

    fn project(&self, v: [f32; 3], center: Pos2, f: f32) -> Option<Pos2> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let x = v[0] * cy - v[2] * sy;
        let z0 = v[0] * sy + v[2] * cy;
        let y = v[1] * cp - z0 * sp;
        let z = v[1] * sp + z0 * cp;
        let depth = self.dist - z;
        if depth < 0.2 {
            return None;
        }
        Some(Pos2::new(center.x + x * f / depth, center.y - y * f / depth))
    }

    pub fn tools(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        theme::card(ui, p, "View", |ui| {
            for (label, yaw, pitch) in [("Perspective", 0.6, 0.45), ("Front", 0.0, 0.0), ("Side", std::f32::consts::FRAC_PI_2, 0.0), ("Top", 0.0, 1.5)] {
                if ui.add_sized([ui.available_width(), 28.0], theme::toggle_button(p, label, (self.yaw - yaw).abs() < 1e-3 && (self.pitch - pitch).abs() < 1e-3)).clicked() {
                    self.yaw = yaw;
                    self.pitch = pitch;
                }
            }
            ui.checkbox(&mut self.grid, "Show grid");
        });
        theme::card(ui, p, "Create", |ui| {
            for (label, kind) in [("Cube", Kind::Cube), ("Joint", Kind::Joint), ("Light", Kind::Light), ("Camera", Kind::Camera)] {
                if ui.add_sized([ui.available_width(), 28.0], theme::toggle_button(p, &format!("{}  {}", kind.icon(), label), false)).clicked() {
                    bus.push(AppCommand::SceneAdd { kind: label.to_lowercase() });
                }
            }
        });
        theme::card(ui, p, "", |ui| {
            ui.label(theme::muted(p, "Wireframe only (no materials or renderer yet). Pick Select/Move/Rotate/Scale on the shelf below; Select-drag or Alt-drag orbits, Shift+Move drags Y, scroll dollies."));
        });
    }

    pub fn center(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = resp.rect;
        painter.rect_filled(rect, Rounding::ZERO, p.canvas);
        let alt = ui.input(|i| i.modifiers.alt);
        let shift = ui.input(|i| i.modifiers.shift);
        let orbit = resp.dragged_by(egui::PointerButton::Middle) || (resp.dragged_by(egui::PointerButton::Primary) && (alt || self.gizmo == Gizmo::Select));
        if orbit {
            let d = resp.drag_delta();
            self.yaw += d.x * 0.008;
            self.pitch = (self.pitch + d.y * 0.008).clamp(-1.5, 1.5);
        } else if resp.dragged_by(egui::PointerButton::Primary) {
            if let Some(i) = self.selected.filter(|i| *i < self.objs.len()) {
                let d = resp.drag_delta();
                let k = self.dist / (rect.height() * 1.1);
                let (sy, cy) = self.yaw.sin_cos();
                let t = self.clock.time;
                let o = &mut self.objs[i];
                match self.gizmo {
                    Gizmo::Move => {
                        if shift {
                            o.pos[1] -= d.y * k;
                            if !o.track_y.is_empty() {
                                o.track_y.set(t, o.pos[1]);
                            }
                        } else {
                            o.pos[0] += (d.x * cy + d.y * sy) * k;
                            o.pos[2] += (-d.x * sy + d.y * cy) * k;
                        }
                    }
                    Gizmo::Rotate => o.rot_y = (o.rot_y + d.x * 0.6) % 360.0,
                    Gizmo::Scale => o.scale = (o.scale * (1.0 + d.x * 0.01)).clamp(0.05, 20.0),
                    Gizmo::Select => {}
                }
            }
        }
        if resp.hovered() {
            let s = ui.input(|i| i.raw_scroll_delta.y);
            if s != 0.0 {
                self.dist = (self.dist * (1.0 - s * 0.0015)).clamp(2.0, 60.0);
            }
            let c = match self.gizmo {
                Gizmo::Select => if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Default },
                Gizmo::Move => egui::CursorIcon::Move,
                Gizmo::Rotate => egui::CursorIcon::Alias,
                Gizmo::Scale => egui::CursorIcon::ResizeHorizontal,
            };
            ui.ctx().set_cursor_icon(c);
        }
        let center = rect.center();
        let f = rect.height() * 1.1;
        let line = |a: [f32; 3], b: [f32; 3], s: Stroke| {
            if let (Some(pa), Some(pb)) = (self.project(a, center, f), self.project(b, center, f)) {
                painter.line_segment([pa, pb], s);
            }
        };
        if self.grid {
            let gs = Stroke::new(1.0_f32, p.border.gamma_multiply(0.8));
            for i in -5..=5 {
                let v = i as f32;
                line([v, 0.0, -5.0], [v, 0.0, 5.0], gs);
                line([-5.0, 0.0, v], [5.0, 0.0, v], gs);
            }
            line([0.0, 0.0, 0.0], [1.5, 0.0, 0.0], Stroke::new(2.0_f32, Color32::from_rgb(230, 80, 80)));
            line([0.0, 0.0, 0.0], [0.0, 1.5, 0.0], Stroke::new(2.0_f32, Color32::from_rgb(90, 200, 100)));
            line([0.0, 0.0, 0.0], [0.0, 0.0, 1.5], Stroke::new(2.0_f32, Color32::from_rgb(90, 140, 240)));
        }
        let mut best: Option<(usize, f32)> = None;
        let click = if resp.clicked() { resp.interact_pointer_pos() } else { None };
        for (i, o) in self.objs.iter().enumerate() {
            if !o.visible {
                continue;
            }
            let sel = self.selected == Some(i);
            let col = if sel { p.accent } else { p.text.gamma_multiply(0.8) };
            let st = Stroke::new(if sel { 2.0_f32 } else { 1.2 }, col);
            let c = o.pos;
            match o.kind {
                Kind::Cube => {
                    if let Some(m) = &o.mesh {
                        let (s, co) = o.rot_y.to_radians().sin_cos();
                        let k = o.scale;
                        let w: Vec<[f32; 3]> = m.verts.iter().map(|v| [c[0] + (v[0] * co - v[2] * s) * k, c[1] + v[1] * k, c[2] + (v[0] * s + v[2] * co) * k]).collect();
                        for (a, b) in &m.edges {
                            line(w[*a], w[*b], st);
                        }
                    }
                }
                Kind::Joint => {
                    if let Some(pp) = o.parent.and_then(|pi| self.objs.get(pi)) {
                        line(pp.pos, c, Stroke::new(2.0_f32, col.gamma_multiply(0.7)));
                    }
                    if let Some(sp) = self.project(c, center, f) {
                        painter.circle_stroke(sp, 6.0, st);
                    }
                }
                Kind::Light => {
                    if let Some(sp) = self.project(c, center, f) {
                        painter.circle_filled(sp, 5.0, Color32::from_rgb(255, 214, 10));
                        for k in 0..8 {
                            let a = k as f32 * std::f32::consts::FRAC_PI_4;
                            let d = Vec2::new(a.cos(), a.sin());
                            painter.line_segment([sp + d * 8.0, sp + d * 13.0], Stroke::new(1.2_f32, col));
                        }
                    }
                }
                Kind::Camera => {
                    let tip = [c[0], c[1], c[2] - 0.8];
                    for (dx, dy) in [(-0.3, -0.2), (0.3, -0.2), (0.3, 0.2), (-0.3, 0.2)] {
                        line(c, [tip[0] + dx, tip[1] + dy, tip[2]], st);
                    }
                    line([tip[0] - 0.3, tip[1] - 0.2, tip[2]], [tip[0] + 0.3, tip[1] - 0.2, tip[2]], st);
                    line([tip[0] + 0.3, tip[1] - 0.2, tip[2]], [tip[0] + 0.3, tip[1] + 0.2, tip[2]], st);
                    line([tip[0] + 0.3, tip[1] + 0.2, tip[2]], [tip[0] - 0.3, tip[1] + 0.2, tip[2]], st);
                    line([tip[0] - 0.3, tip[1] + 0.2, tip[2]], [tip[0] - 0.3, tip[1] - 0.2, tip[2]], st);
                }
            }
            if let (Some(cp), Some(sp)) = (click, self.project(c, center, f)) {
                let d = cp.distance(sp);
                if d < 28.0 && best.map(|b| d < b.1).unwrap_or(true) {
                    best = Some((i, d));
                }
            }
            if let Some(sp) = self.project(c, center, f) {
                painter.text(sp + Vec2::new(10.0, -10.0), egui::Align2::LEFT_BOTTOM, &o.name, egui::FontId::proportional(11.0), if sel { p.accent } else { p.muted });
            }
        }
        if click.is_some() {
            self.selected = best.map(|b| b.0);
        }
        painter.text(rect.min + Vec2::new(SP_MD, SP_MD), egui::Align2::LEFT_TOP, format!("Perspective  ·  {} objects  ·  {} tool", self.objs.len(), self.gizmo.label()), egui::FontId::proportional(12.0), p.muted);
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        theme::card(ui, p, "Outliner", |ui| {
            let mut pick = None;
            let mut vis = None;
            for (i, o) in self.objs.iter().enumerate() {
                let on = self.selected == Some(i);
                egui::Frame::none().fill(if on { p.accent_soft() } else { Color32::TRANSPARENT }).rounding(Rounding::same(R_SM + 1.0)).inner_margin(egui::Margin::symmetric(SP_XS, 1.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if o.parent.is_some() {
                            ui.add_space(SP_MD);
                        }
                        if ui.add(egui::Button::new(egui::RichText::new(if o.visible { "👁" } else { "–" }).color(p.muted)).frame(false)).clicked() {
                            vis = Some(i);
                        }
                        if ui.add(egui::Button::new(egui::RichText::new(format!("{}  {}", o.kind.icon(), o.name)).color(if on { p.accent } else { p.text })).frame(false)).clicked() {
                            pick = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = vis {
                self.objs[i].visible = !self.objs[i].visible;
            }
            if let Some(i) = pick {
                bus.push(AppCommand::SceneSelect { object: self.objs[i].name.clone() });
            }
            if self.objs.is_empty() {
                ui.label(theme::muted(p, "Empty scene. Create an object on the left."));
            }
        });
        let t = self.clock.time;
        let Some(i) = self.selected.filter(|i| *i < self.objs.len()) else {
            theme::card(ui, p, "Transform", |ui| {
                ui.label(theme::muted(p, "Select an object to edit it."));
            });
            return;
        };
        let mut delete = false;
        theme::card(ui, p, "Transform", |ui| {
            let o = &mut self.objs[i];
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&o.name).strong().color(p.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::toggle_button(p, "Delete", false)).clicked() {
                        delete = true;
                    }
                });
            });
            egui::Grid::new("xf").num_columns(2).spacing([SP_SM, 6.0]).show(ui, |ui| {
                ui.label(theme::muted(p, "Position"));
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut o.pos[0]).speed(0.05).prefix("x ").max_decimals(2));
                    let r = ui.add(egui::DragValue::new(&mut o.pos[1]).speed(0.05).prefix("y ").max_decimals(2));
                    if r.changed() && !o.track_y.is_empty() {
                        o.track_y.set(t, o.pos[1]);
                    }
                    ui.add(egui::DragValue::new(&mut o.pos[2]).speed(0.05).prefix("z ").max_decimals(2));
                });
                ui.end_row();
                ui.label(theme::muted(p, "Rotate Y"));
                ui.add(egui::DragValue::new(&mut o.rot_y).speed(1.0).suffix("°"));
                ui.end_row();
                ui.label(theme::muted(p, "Scale"));
                ui.add(egui::DragValue::new(&mut o.scale).speed(0.02).range(0.05..=20.0));
                ui.end_row();
            });
        });
        theme::card(ui, p, "Animation · Y position", |ui| {
            let o = &mut self.objs[i];
            theme::kv(ui, p, "Keys", format!("{}", o.track_y.keys().len()));
            ui.horizontal(|ui| {
                if ui.add(theme::primary_button(p, "Add key")).on_hover_text("Key the current Y at the playhead  (K)").clicked() {
                    bus.push(AppCommand::SetKeyframe);
                }
                let near = o.track_y.has_key_near(t, 0.05);
                if ui.add_enabled(near, theme::toggle_button(p, "Delete key", false)).clicked() {
                    bus.push(AppCommand::DeleteKeyframe);
                }
            });
            if o.kind == Kind::Joint {
                if let Ok(j) = self.joints.get(&o.name) {
                    let (x, y, z) = j.position();
                    theme::kv(ui, p, "JointHierarchy", format!("{:.2}, {:.2}, {:.2}", x, y, z));
                }
            }
        });
        if delete {
            bus.push(AppCommand::SceneDelete { object: None });
        }
    }

    pub fn timeline(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        ui.horizontal(|ui| timeline::transport(ui, p, &mut self.clock, bus));
        ui.add_space(SP_XS);
        let tracks: Vec<Track> = self.objs.iter().enumerate().filter(|(i, o)| !o.track_y.is_empty() || self.selected == Some(*i)).map(|(i, o)| Track { label: &o.name, keys: &o.track_y, selected: self.selected == Some(i) }).collect();
        let names: Vec<String> = tracks.iter().map(|t| t.label.to_string()).collect();
        let out = timeline::ruler(ui, p, "scene-ruler", &mut self.clock, &tracks);
        if let Some(k) = out.picked_track {
            if let Some(name) = names.get(k) {
                self.selected = self.objs.iter().position(|o| &o.name == name);
            }
        }
    }
}
