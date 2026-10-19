//! Safe pure ToolOptions (options bar for the active tool).
//!
//! Returns the settings the active tool shows.
//! Brush has size and opacity. Crop has ratio.
//! Unknown tools have no options. No unwraps.
//!
//! Differential: a different tool has different options.

use crate::active_tool::ActiveTool;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolOption {
    pub name: &'static str,
    pub value: f32,
}

pub struct ToolOptions;

impl ToolOptions {
    /// Options for the active tool.
    pub fn for_tool(active: &ActiveTool) -> Vec<ToolOption> {
        match active.name() {
            "Brush" | "Pencil" | "Eraser" => vec![
                ToolOption { name: "size", value: 20.0 },
                ToolOption { name: "opacity", value: 100.0 },
                ToolOption { name: "hardness", value: 50.0 },
            ],
            "Clone Stamp" | "Healing Brush" | "Spot Healing" => vec![
                ToolOption { name: "size", value: 20.0 },
                ToolOption { name: "opacity", value: 100.0 },
            ],
            "Crop" => vec![
                ToolOption { name: "ratio_w", value: 0.0 },
                ToolOption { name: "ratio_h", value: 0.0 },
            ],
            "Gradient" => vec![
                ToolOption { name: "opacity", value: 100.0 },
            ],
            "Zoom" => vec![
                ToolOption { name: "percent", value: 100.0 },
            ],
            _ => Vec::new(),
        }
    }

    /// One line of option names.
    pub fn line(active: &ActiveTool) -> String {
        let opts = Self::for_tool(active);
        if opts.is_empty() {
            return format!("{}: (no options)", active.name());
        }
        let names = opts.iter().map(|o| o.name).collect::<Vec<_>>().join(", ");
        format!("{}: {}", active.name(), names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_tool::ToolSource;

    #[test]
    fn brush_has_options() {
        let mut t = ActiveTool::new();
        t.activate("Brush", ToolSource::Photoshop);
        let opts = ToolOptions::for_tool(&t);
        assert!(opts.iter().any(|o| o.name == "size"));
        assert!(opts.iter().any(|o| o.name == "opacity"));
        assert!(ToolOptions::line(&t).contains("size"));
    }

    #[test]
    fn move_has_none() {
        let t = ActiveTool::new();
        assert!(ToolOptions::for_tool(&t).is_empty());
    }

    #[test]
    fn differential() {
        let mut a = ActiveTool::new();
        let mut b = ActiveTool::new();
        a.activate("Brush", ToolSource::Photoshop);
        b.activate("Crop", ToolSource::Photoshop);
        assert_ne!(ToolOptions::line(&a), ToolOptions::line(&b));
    }
}
