//! Maya-style shelf: sub-tabs with big icon buttons that drive the 3D
//! scene through the command bus. Unimplemented tools are shown
//! disabled with a "Coming soon" tooltip.

use crate::commands::AppCommand;
use crate::scene::Gizmo;
use crate::theme::{self, Palette, SP_SM, SP_XS};
use eframe::egui::{self, Pos2, Rounding, Stroke, Vec2};
use egui_phosphor::regular as ic;

pub const TABS: [&str; 5] = ["Modeling", "Rigging", "Animation", "FX", "Rendering"];

enum Item {
    Btn { icon: &'static str, label: &'static str, tip: &'static str, cmds: fn() -> Vec<AppCommand>, gizmo: Option<Gizmo> },
    Soon { icon: &'static str, label: &'static str },
    Sep,
}

fn g(mode: &'static str) -> Vec<AppCommand> {
    vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::SceneGizmo { mode: mode.into() }]
}

fn items(tab: usize) -> Vec<Item> {
    use Item::*;
    match tab {
        0 => vec![
            Btn { icon: ic::CURSOR, label: "Select", tip: "Select objects; drag orbits  (Q)", cmds: || g("select"), gizmo: Some(Gizmo::Select) },
            Btn { icon: ic::ARROWS_OUT_CARDINAL, label: "Move", tip: "Drag to move on the ground plane, Shift for Y  (W)", cmds: || g("move"), gizmo: Some(Gizmo::Move) },
            Btn { icon: ic::ARROW_CLOCKWISE, label: "Rotate", tip: "Drag to rotate around Y  (E)", cmds: || g("rotate"), gizmo: Some(Gizmo::Rotate) },
            Btn { icon: ic::ARROWS_OUT, label: "Scale", tip: "Drag to scale uniformly  (R)", cmds: || g("scale"), gizmo: Some(Gizmo::Scale) },
            Sep,
            Btn { icon: ic::CUBE, label: "Cube", tip: "Create a polygon cube", cmds: || vec![AppCommand::SceneAdd { kind: "cube".into() }], gizmo: None },
            Btn { icon: ic::ARROW_FAT_LINE_UP, label: "Extrude", tip: "Extrude the selected cube's top face by 0.5", cmds: || vec![AppCommand::SceneExtrude { distance: Some(0.5) }], gizmo: None },
            Btn { icon: ic::POLYGON, label: "Bevel", tip: "Chamfer the selected cube's top face", cmds: || vec![AppCommand::SceneBevel { amount: Some(0.15) }], gizmo: None },
            Sep,
            Soon { icon: ic::DOTS_THREE_OUTLINE, label: "Edge Loop" },
            Soon { icon: ic::KNIFE, label: "Multi-Cut" },
            Soon { icon: ic::BRIDGE, label: "Bridge" },
            Soon { icon: ic::INTERSECT, label: "Boolean" },
            Soon { icon: ic::GRID_FOUR, label: "Quad Draw" },
            Sep,
            Btn { icon: ic::TRASH, label: "Delete", tip: "Delete the selected object  (Del)", cmds: || vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::SceneDelete { object: None }], gizmo: None },
        ],
        1 => vec![
            Btn { icon: ic::BONE, label: "Joint", tip: "Create a joint (JointHierarchy)", cmds: || vec![AppCommand::SceneAdd { kind: "joint".into() }], gizmo: None },
            Btn { icon: ic::ARROWS_OUT_CARDINAL, label: "Move", tip: "Pose with the Move tool", cmds: || g("move"), gizmo: Some(Gizmo::Move) },
            Btn { icon: ic::ARROW_CLOCKWISE, label: "Rotate", tip: "Pose with the Rotate tool", cmds: || g("rotate"), gizmo: Some(Gizmo::Rotate) },
            Sep,
            Soon { icon: ic::LINK, label: "IK Handle" },
            Soon { icon: ic::PERSON, label: "Skin Bind" },
            Soon { icon: ic::PAINT_BRUSH, label: "Paint Weights" },
        ],
        2 => vec![
            Btn { icon: ic::DIAMOND, label: "Set Key", tip: "Key the selected object's Y at the playhead  (K)", cmds: || vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::SetKeyframe], gizmo: None },
            Btn { icon: ic::X_SQUARE, label: "Delete Key", tip: "Delete the key at the playhead", cmds: || vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::DeleteKeyframe], gizmo: None },
            Sep,
            Btn { icon: ic::SKIP_BACK, label: "Start", tip: "Go to the first frame", cmds: || vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::TimelineScrub { time: 0.0 }], gizmo: None },
            Btn { icon: ic::PLAY, label: "Play", tip: "Play / pause  (Space)", cmds: || vec![AppCommand::SwitchMode { mode: "3d".into() }, AppCommand::TimelinePlay { playing: None }], gizmo: None },
            Sep,
            Soon { icon: ic::CHART_LINE, label: "Graph Editor" },
            Soon { icon: ic::FILM_SLATE, label: "Playblast" },
        ],
        3 => vec![Soon { icon: ic::SPARKLE, label: "Particles" }, Soon { icon: ic::DROP, label: "Fluids" }, Soon { icon: ic::WIND, label: "Fields" }, Soon { icon: ic::FLAG, label: "nCloth" }],
        _ => vec![
            Btn { icon: ic::LIGHTBULB, label: "Light", tip: "Create a point light", cmds: || vec![AppCommand::SceneAdd { kind: "light".into() }], gizmo: None },
            Btn { icon: ic::VIDEO_CAMERA, label: "Camera", tip: "Create a camera", cmds: || vec![AppCommand::SceneAdd { kind: "camera".into() }], gizmo: None },
            Sep,
            Soon { icon: ic::IMAGE, label: "Render" },
            Soon { icon: ic::PALETTE, label: "Materials" },
            Soon { icon: ic::SUN_HORIZON, label: "Sky Dome" },
        ],
    }
}

/// Draw the shelf. `active` is the current gizmo when in 3D mode.
pub fn shelf(ui: &mut egui::Ui, p: &Palette, tab: &mut usize, active: Option<Gizmo>, bus: &mut Vec<AppCommand>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (i, t) in TABS.iter().enumerate() {
            let on = *tab == i;
            let font = egui::FontId::proportional(11.5);
            let w = ui.painter().layout_no_wrap(t.to_string(), font.clone(), p.text).size().x + 24.0;
            let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 22.0), egui::Sense::click());
            if on {
                ui.painter().rect(r, Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 }, p.surface_hi, Stroke::new(1.0_f32, p.line));
                ui.painter().hline(r.x_range(), r.bottom() - 1.0, Stroke::new(2.0_f32, p.accent));
            } else if resp.hovered() {
                ui.painter().rect_filled(r, Rounding::same(4.0), p.surface);
            }
            ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, *t, font, if on { p.accent } else { p.muted });
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *tab = i;
            }
        }
    });
    let r = ui.min_rect();
    ui.painter().hline(r.x_range(), ui.cursor().top(), Stroke::new(1.0_f32, p.border));
    ui.add_space(SP_XS);
    egui::ScrollArea::horizontal().id_salt("shelf").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SP_XS;
            for it in items(*tab) {
                match it {
                    Item::Btn { icon, label, tip, cmds, gizmo } => {
                        let on = gizmo.is_some() && gizmo == active;
                        if theme::big_tool(ui, p, icon, label, on, true).on_hover_text(tip).clicked() {
                            bus.extend(cmds());
                        }
                    }
                    Item::Soon { icon, label } => {
                        theme::big_tool(ui, p, icon, label, false, false).on_hover_text(format!("{} · Coming soon", label));
                    }
                    Item::Sep => {
                        let (r, _) = ui.allocate_exact_size(Vec2::new(SP_SM + 1.0, 54.0), egui::Sense::hover());
                        ui.painter().line_segment([Pos2::new(r.center().x, r.top() + 8.0), Pos2::new(r.center().x, r.bottom() - 8.0)], Stroke::new(1.0_f32, p.border));
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_enabled_shelf_button_dispatches_a_3d_command() {
        for t in 0..TABS.len() {
            for it in items(t) {
                if let Item::Btn { cmds, .. } = it {
                    let c = cmds();
                    assert!(!c.is_empty());
                    assert!(c.iter().all(|c| c.name().starts_with("scene") || matches!(c, AppCommand::SwitchMode { .. } | AppCommand::SetKeyframe | AppCommand::DeleteKeyframe | AppCommand::TimelineScrub { .. } | AppCommand::TimelinePlay { .. })));
                }
            }
        }
    }
}
