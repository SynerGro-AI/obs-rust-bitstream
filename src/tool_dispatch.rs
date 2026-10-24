//! Safe pure ToolDispatch (active tool routing).
//!
//! Maps a tool name to an action. The canvas calls
//! dispatch on click. No unwraps.
//!
//! Differential: a different tool produces a different action.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasAction {
    Paint { x: u32, y: u32 },
    Select { x: u32, y: u32 },
    Crop { x: u32, y: u32 },
    Heal { x: u32, y: u32 },
    None,
}

pub struct ToolDispatch;

impl ToolDispatch {
    /// Route a tool name and click to an action.
    pub fn dispatch(tool: &str, x: u32, y: u32) -> CanvasAction {
        match tool {
            "Brush" | "Brush Tool" | "Pen Tool" | "Pen" | "Eraser Tool" | "Eraser"
            | "Gradient Tool" => CanvasAction::Paint { x, y },
            "Lasso" | "Lasso Tool" | "Rectangular Marquee" | "Quick Select" => {
                CanvasAction::Select { x, y }
            }
            "Crop" | "Crop Tool" => CanvasAction::Crop { x, y },
            "Healing Brush" | "Heal" | "Clone Stamp" => CanvasAction::Heal { x, y },
            "Move Tool" | "Move" | "Hand Tool" | "Hand" | "Zoom Tool" | "Zoom" => {
                CanvasAction::None
            }
            _ => CanvasAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint() {
        let a = ToolDispatch::dispatch("Brush", 10, 20);
        assert_eq!(a, CanvasAction::Paint { x: 10, y: 20 });
    }

    #[test]
    fn select() {
        let a = ToolDispatch::dispatch("Lasso", 5, 5);
        assert_eq!(a, CanvasAction::Select { x: 5, y: 5 });
    }

    #[test]
    fn none() {
        let a = ToolDispatch::dispatch("Unknown", 0, 0);
        assert_eq!(a, CanvasAction::None);
    }

    #[test]
    fn differential() {
        let a = ToolDispatch::dispatch("Brush", 1, 1);
        let b = ToolDispatch::dispatch("Crop", 1, 1);
        assert_ne!(a, b);
    }
}
