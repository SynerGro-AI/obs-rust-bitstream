//! Safe pure PsToolbar (Photoshop toolbar layout).
//!
//! Groups and hotkeys match the Photoshop toolbar order.
//! Tools map to existing suite modules where present.
//! No unwraps.
//!
//! Differential: a different group has a different tool count.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PsGroup {
    Move,
    Selection,
    Crop,
    Measure,
    Retouch,
    Paint,
    Draw,
    Type,
    Navigate,
}

impl PsGroup {
    pub fn label(&self) -> &'static str {
        match self {
            PsGroup::Move => "Move",
            PsGroup::Selection => "Selection",
            PsGroup::Crop => "Crop",
            PsGroup::Measure => "Measure",
            PsGroup::Retouch => "Retouch",
            PsGroup::Paint => "Paint",
            PsGroup::Draw => "Draw",
            PsGroup::Type => "Type",
            PsGroup::Navigate => "Navigate",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PsTool {
    pub name: &'static str,
    pub hotkey: &'static str,
    pub group: PsGroup,
    /// Suite module name if one exists, otherwise None.
    pub module: Option<&'static str>,
}

pub struct PsToolbar;

impl PsToolbar {
    /// The full toolbar in Photoshop order.
    pub fn tools() -> Vec<PsTool> {
        vec![
            PsTool { name: "Move", hotkey: "V", group: PsGroup::Move, module: None },
            PsTool { name: "Rectangular Marquee", hotkey: "M", group: PsGroup::Selection, module: Some("selection") },
            PsTool { name: "Elliptical Marquee", hotkey: "M", group: PsGroup::Selection, module: Some("selection") },
            PsTool { name: "Lasso", hotkey: "L", group: PsGroup::Selection, module: Some("lasso") },
            PsTool { name: "Quick Selection", hotkey: "W", group: PsGroup::Selection, module: Some("quick_select") },
            PsTool { name: "Crop", hotkey: "C", group: PsGroup::Crop, module: Some("crop") },
            PsTool { name: "Eyedropper", hotkey: "I", group: PsGroup::Measure, module: None },
            PsTool { name: "Spot Healing", hotkey: "J", group: PsGroup::Retouch, module: Some("heal") },
            PsTool { name: "Healing Brush", hotkey: "J", group: PsGroup::Retouch, module: Some("heal") },
            PsTool { name: "Clone Stamp", hotkey: "S", group: PsGroup::Retouch, module: Some("clone_stamp") },
            PsTool { name: "Brush", hotkey: "B", group: PsGroup::Paint, module: Some("brush") },
            PsTool { name: "Eraser", hotkey: "E", group: PsGroup::Paint, module: Some("eraser") },
            PsTool { name: "Gradient", hotkey: "G", group: PsGroup::Paint, module: Some("gradient") },
            PsTool { name: "Paint Bucket", hotkey: "G", group: PsGroup::Paint, module: Some("paint_bucket") },
            PsTool { name: "Dodge", hotkey: "O", group: PsGroup::Paint, module: Some("dodge_burn_sponge") },
            PsTool { name: "Burn", hotkey: "O", group: PsGroup::Paint, module: Some("dodge_burn_sponge") },
            PsTool { name: "Sponge", hotkey: "O", group: PsGroup::Paint, module: Some("dodge_burn_sponge") },
            PsTool { name: "Pen", hotkey: "P", group: PsGroup::Draw, module: None },
            PsTool { name: "Horizontal Type", hotkey: "T", group: PsGroup::Type, module: None },
            PsTool { name: "Rectangle", hotkey: "U", group: PsGroup::Draw, module: Some("shape") },
            PsTool { name: "Hand", hotkey: "H", group: PsGroup::Navigate, module: Some("hand") },
            PsTool { name: "Zoom", hotkey: "Z", group: PsGroup::Navigate, module: None },
        ]
    }

    pub fn groups() -> &'static [PsGroup] {
        &[
            PsGroup::Move,
            PsGroup::Selection,
            PsGroup::Crop,
            PsGroup::Measure,
            PsGroup::Retouch,
            PsGroup::Paint,
            PsGroup::Draw,
            PsGroup::Type,
            PsGroup::Navigate,
        ]
    }

    /// Tools in one group.
    pub fn in_group(group: PsGroup) -> Vec<PsTool> {
        Self::tools().into_iter().filter(|t| t.group == group).collect()
    }

    /// One line of tool names.
    pub fn line() -> String {
        Self::tools()
            .iter()
            .map(|t| t.name)
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_core_tools() {
        let tools = PsToolbar::tools();
        assert!(tools.iter().any(|t| t.name == "Brush" && t.hotkey == "B"));
        assert!(tools.iter().any(|t| t.name == "Lasso" && t.module == Some("lasso")));
        assert!(tools.iter().any(|t| t.name == "Crop" && t.module == Some("crop")));
    }

    #[test]
    fn groups_present() {
        assert_eq!(PsToolbar::groups().len(), 9);
        assert!(PsToolbar::in_group(PsGroup::Paint).len() >= 5);
    }

    #[test]
    fn differential() {
        let paint = PsToolbar::in_group(PsGroup::Paint);
        let move_g = PsToolbar::in_group(PsGroup::Move);
        assert!(paint.len() > move_g.len());
    }
}
