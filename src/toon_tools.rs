//! Safe pure ToonTools (ToonBoom Harmony tools toolbar).
//!
//! Tools match the Harmony Tools toolbar and Onion Skin
//! toolbar. Maps to suite modules where present.
//! No unwraps.
//!
//! Differential: onion skin tools are a subset.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToonGroup {
    Tools,
    Onion,
}

impl ToonGroup {
    pub fn label(&self) -> &'static str {
        match self {
            ToonGroup::Tools => "Tools",
            ToonGroup::Onion => "Onion Skin",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToonTool {
    pub name: &'static str,
    pub hotkey: &'static str,
    pub group: ToonGroup,
    pub module: Option<&'static str>,
}

pub struct ToonTools;

impl ToonTools {
    pub fn tools() -> Vec<ToonTool> {
        vec![
            ToonTool {
                name: "Select",
                hotkey: "",
                group: ToonGroup::Tools,
                module: Some("selection"),
            },
            ToonTool {
                name: "Brush",
                hotkey: "",
                group: ToonGroup::Tools,
                module: Some("brush"),
            },
            ToonTool {
                name: "Pencil",
                hotkey: "",
                group: ToonGroup::Tools,
                module: None,
            },
            ToonTool {
                name: "Eraser",
                hotkey: "",
                group: ToonGroup::Tools,
                module: Some("eraser"),
            },
            ToonTool {
                name: "Hand",
                hotkey: "",
                group: ToonGroup::Tools,
                module: Some("hand"),
            },
            ToonTool {
                name: "Zoom",
                hotkey: "",
                group: ToonGroup::Tools,
                module: None,
            },
            ToonTool {
                name: "Rotate View",
                hotkey: "",
                group: ToonGroup::Tools,
                module: None,
            },
            ToonTool {
                name: "Transform",
                hotkey: "",
                group: ToonGroup::Tools,
                module: None,
            },
            ToonTool {
                name: "Onion Skin",
                hotkey: "Alt+O",
                group: ToonGroup::Tools,
                module: Some("onion_viewer"),
            },
            ToonTool {
                name: "Show Key Drawings",
                hotkey: "",
                group: ToonGroup::Onion,
                module: Some("onion_viewer"),
            },
            ToonTool {
                name: "Show Breakdown",
                hotkey: "",
                group: ToonGroup::Onion,
                module: Some("onion_viewer"),
            },
            ToonTool {
                name: "Show In-between",
                hotkey: "",
                group: ToonGroup::Onion,
                module: Some("onion_viewer"),
            },
        ]
    }

    pub fn in_group(group: ToonGroup) -> Vec<ToonTool> {
        Self::tools()
            .into_iter()
            .filter(|t| t.group == group)
            .collect()
    }

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
    fn has_core() {
        let tools = ToonTools::tools();
        assert!(tools
            .iter()
            .any(|t| t.name == "Onion Skin" && t.hotkey == "Alt+O"));
        assert!(tools
            .iter()
            .any(|t| t.name == "Brush" && t.module == Some("brush")));
    }

    #[test]
    fn onion_subset() {
        assert!(ToonTools::in_group(ToonGroup::Onion).len() >= 3);
    }

    #[test]
    fn differential() {
        let tools = ToonTools::in_group(ToonGroup::Tools);
        let onion = ToonTools::in_group(ToonGroup::Onion);
        assert!(tools.len() > onion.len());
    }
}
