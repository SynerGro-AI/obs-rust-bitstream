//! Safe pure AeTools (After Effects tools panel).
//!
//! Tools and hotkeys match the AE tools panel.
//! No unwraps.
//!
//! Differential: the panel has more tools than a single group.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AeTool {
    pub name: &'static str,
    pub hotkey: &'static str,
}

pub struct AeTools;

impl AeTools {
    pub fn tools() -> Vec<AeTool> {
        vec![
            AeTool {
                name: "Selection",
                hotkey: "V",
            },
            AeTool {
                name: "Hand",
                hotkey: "H",
            },
            AeTool {
                name: "Zoom",
                hotkey: "Z",
            },
            AeTool {
                name: "Rotation",
                hotkey: "W",
            },
            AeTool {
                name: "Pan Behind",
                hotkey: "Y",
            },
            AeTool {
                name: "Camera",
                hotkey: "C",
            },
            AeTool {
                name: "Shape",
                hotkey: "Q",
            },
            AeTool {
                name: "Pen",
                hotkey: "G",
            },
            AeTool {
                name: "Text",
                hotkey: "Ctrl+T",
            },
            AeTool {
                name: "Brush",
                hotkey: "Ctrl+B",
            },
            AeTool {
                name: "Clone Stamp",
                hotkey: "Ctrl+B",
            },
            AeTool {
                name: "Eraser",
                hotkey: "Ctrl+B",
            },
            AeTool {
                name: "Roto Brush",
                hotkey: "Alt+W",
            },
            AeTool {
                name: "Puppet Pin",
                hotkey: "Ctrl+P",
            },
        ]
    }

    pub fn line() -> String {
        Self::tools()
            .iter()
            .map(|t| format!("{} ({})", t.name, t.hotkey))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    pub fn by_hotkey(hotkey: &str) -> Vec<AeTool> {
        Self::tools()
            .into_iter()
            .filter(|t| t.hotkey == hotkey)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_core() {
        let tools = AeTools::tools();
        assert!(tools
            .iter()
            .any(|t| t.name == "Selection" && t.hotkey == "V"));
        assert!(tools.iter().any(|t| t.name == "Pen" && t.hotkey == "G"));
        assert!(tools.iter().any(|t| t.name == "Camera"));
    }

    #[test]
    fn shared_hotkey() {
        // Brush, Clone, Eraser share Ctrl+B in AE.
        assert_eq!(AeTools::by_hotkey("Ctrl+B").len(), 3);
    }

    #[test]
    fn differential() {
        assert!(AeTools::tools().len() > 10);
        assert!(AeTools::line().contains("Selection (V)"));
    }
}
