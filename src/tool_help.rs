//! Safe pure ToolHelp (💭 instructions for tools and options).
//!
//! When a tool or option is selected, returns an
//! instructive thought that explains how to use it.
//! No unwraps.
//!
//! Differential: a different option produces a different thought.

use crate::active_tool::ActiveTool;

pub struct ToolHelp;

impl ToolHelp {
    /// Thought for the active tool.
    pub fn tool(name: &str) -> &'static str {
        match name {
            "Brush" => "💭 Brush paints on the selected layer. Drag to draw. Size and opacity control the stroke. Hold Alt to sample a color.",
            "Pencil" => "💭 Pencil draws hard-edged strokes. Size sets the width. Opacity sets how strong the mark is.",
            "Eraser" => "💭 Eraser removes pixels on the selected layer. Size controls the area. Opacity controls how much is removed.",
            "Clone Stamp" => "💭 Clone Stamp copies pixels from a source point. Alt-click to set the source, then paint to clone.",
            "Healing Brush" => "💭 Healing Brush blends the source into the destination. Alt-click to set the source, then paint.",
            "Crop" => "💭 Crop trims the canvas. Drag the handles to set the area, then press Enter to apply.",
            "Move" => "💭 Move repositions the selected layer. Drag to shift. Arrow keys nudge by one pixel.",
            "Zoom" => "💭 Zoom changes the view scale. Click to zoom in. Alt-click to zoom out. Percent sets the exact scale.",
            "Gradient" => "💭 Gradient fills with a blend. Drag to set the direction. Opacity controls the strength.",
            "Lasso" => "💭 Lasso selects a freehand region. Drag to draw the outline, then release to close it.",
            "Marquee" => "💭 Marquee selects a rectangular region. Drag from corner to corner.",
            _ => "💭 Select a tool from the toolbar. Its options appear above the canvas.",
        }
    }

    /// Thought for a specific option on a tool.
    pub fn option(tool: &str, option: &str) -> &'static str {
        match (tool, option) {
            ("Brush", "size") | ("Pencil", "size") | ("Eraser", "size") => {
                "💭 Size sets the brush diameter in pixels. Larger covers more area."
            }
            ("Brush", "opacity") | ("Pencil", "opacity") | ("Eraser", "opacity") => {
                "💭 Opacity sets how strong the stroke is. 100 is solid. Lower is translucent."
            }
            ("Brush", "hardness") => {
                "💭 Hardness sets the edge falloff. 100 is a hard edge. Lower is softer."
            }
            ("Crop", "ratio_w") | ("Crop", "ratio_h") => {
                "💭 Ratio locks the crop to a width-to-height proportion. 0 is free."
            }
            ("Zoom", "percent") => {
                "💭 Percent sets the exact view scale. 100 is actual size."
            }
            ("Gradient", "opacity") => {
                "💭 Opacity sets how strong the gradient fill is."
            }
            _ => "💭 Adjust this option to change how the tool behaves.",
        }
    }

    /// Thought for the currently selected option, or the tool if none.
    pub fn for_active(active: &ActiveTool, selected_option: Option<&str>) -> &'static str {
        if let Some(opt) = selected_option {
            Self::option(active.name(), opt)
        } else {
            Self::tool(active.name())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_tool::ToolSource;

    #[test]
    fn tool_thought() {
        let t = ToolHelp::tool("Brush");
        assert!(t.contains("💭"));
        assert!(t.contains("Brush"));
    }

    #[test]
    fn option_thought() {
        let t = ToolHelp::option("Brush", "size");
        assert!(t.contains("💭"));
        assert!(t.contains("Size"));
    }

    #[test]
    fn for_active() {
        let mut a = ActiveTool::new();
        a.activate("Brush", ToolSource::Photoshop);
        let t = ToolHelp::for_active(&a, Some("opacity"));
        assert!(t.contains("Opacity"));
    }

    #[test]
    fn differential() {
        let a = ToolHelp::option("Brush", "size");
        let b = ToolHelp::option("Brush", "opacity");
        assert_ne!(a, b);
    }
}
