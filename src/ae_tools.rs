//! Safe pure AeTools (After Effects tools panel).
//!
//! Tools and hotkeys match the AE tools panel.
//! Maps to suite modules where present. No unwraps.
//!
//! Differential: the panel has more tools than a single group.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AeTool {
    pub name: &'static str,
    pub hotkey: &'static str,
    pub module: Option<&'static str>,
}

pub struct AeTools;

impl AeTools {
    pub fn tools() -> Vec<AeTool> {
        vec![
            AeTool { name: "Selection", hotkey: "V", module: Some("selection") },
            AeTool { name: "Hand", hotkey: "H", module: Some("hand") },
            AeTool { name: "Zoom", hotkey: "Z", module: None },
            AeTool { name: "Rotation", hotkey: "W", module: None },
            AeTool { name: "Pan Behind", hotkey: "Y", module: None },
            AeTool { name: "Camera", hotkey: "C", module: Some("camera") },
            AeTool { name: "Shape", hotkey: "Q", module: Some("shape") },
            AeTool { name: "Pen", hotkey: "G", module: None },
            AeTool { name: "Text", hotkey: "Ctrl+T", module: None },
            AeTool { name: "Brush", hotkey: "Ctrl+B", module: Some("brush") },
            AeTool { name: "Clone Stamp", hotkey: "Ctrl+B", module: Some("clone_stamp") },
            AeTool { name: "Eraser", hotkey: "Ctrl+B", module: Some("eraser") },
            AeTool { name: "Roto Brush", hotkey: "Alt+W", module: None },
            AeTool { name: "Puppet Pin", hotkey: "Ctrl+P", module: None },
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
        assert!(tools.iter().any(|t| t.name == "Selection" && t.hotkey == "V"));
        assert!(tools.iter().any(|t| t.name == "Pen" && t.hotkey == "G"));
        assert!(tools.iter().any(|t| t.name == "Camera" && t.module == Some("camera")));
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
