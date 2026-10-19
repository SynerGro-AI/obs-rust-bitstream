//! Safe pure StatusBar (bottom bar for the shell).
//!
//! Shows the active tool, zoom, canvas size, and a 💭 thought.
//! One line. No unwraps.
//!
//! Differential: a different tool produces a different line.

use crate::tool_help::ToolHelp;

pub struct StatusBar;

impl StatusBar {
    /// One line: tool, zoom, canvas, thought.
    pub fn line(tool: &str, zoom: f32, width: u32, height: u32) -> String {
        let thought = ToolHelp::tool(tool);
        format!("tool={} zoom={:.0}% canvas={}x{} | {}", tool, zoom, width, height, thought)
    }

    /// True if the line has every section.
    pub fn complete(line: &str) -> bool {
        line.contains("tool=") && line.contains("zoom=") && line.contains("canvas=") && line.contains("💭")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sections() {
        let line = StatusBar::line("Brush", 100.0, 1920, 1080);
        assert!(StatusBar::complete(&line));
        assert!(line.contains("Brush"));
        assert!(line.contains("1920x1080"));
    }

    #[test]
    fn differential() {
        let a = StatusBar::line("Brush", 100.0, 1920, 1080);
        let b = StatusBar::line("Crop", 50.0, 1920, 1080);
        assert_ne!(a, b);
    }
}
