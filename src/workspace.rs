//! Safe pure Workspace (one canvas, all tools).
//!
//! Every tool from Photoshop, Maya, AE, and ToonBoom
//! works on the same canvas. No workspace switch.
//! The active tool is just a name. No unwraps.
//!
//! Differential: a different active tool changes the name.

use crate::ae_tools::AeTools;
use crate::maya_shelves::MayaShelves;
use crate::pixel_canvas::PixelCanvas;
use crate::ps_toolbar::PsToolbar;
use crate::toon_tools::ToonTools;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    active: String,
    width: u32,
    height: u32,
}

impl Workspace {
    /// One canvas. All tools available.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            active: "Move".to_string(),
            width,
            height,
        }
    }

    pub fn active(&self) -> &str {
        &self.active
    }

    /// Set the active tool by name. Any tool from any toolbar.
    /// Unknown names are rejected.
    pub fn set_tool(&mut self, name: &str) -> bool {
        if Self::all_tools().iter().any(|t| t == name) {
            self.active = name.to_string();
            true
        } else {
            false
        }
    }

    /// Every tool name from all four toolbars.
    pub fn all_tools() -> Vec<String> {
        let mut names = Vec::new();
        for t in PsToolbar::tools() {
            names.push(t.name.to_string());
        }
        for b in MayaShelves::buttons() {
            names.push(b.name.to_string());
        }
        for t in AeTools::tools() {
            names.push(t.name.to_string());
        }
        for t in ToonTools::tools() {
            names.push(t.name.to_string());
        }
        names
    }

    /// The shared canvas. Same for every tool.
    pub fn canvas(&self) -> Option<PixelCanvas> {
        PixelCanvas::new(self.width, self.height).ok()
    }

    /// One line: active tool and canvas size.
    pub fn line(&self) -> String {
        format!("tool={} canvas={}x{}", self.active, self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_canvas() {
        let mut w = Workspace::new(64, 64);
        assert!(w.set_tool("Brush"));
        assert_eq!(w.active(), "Brush");
        assert!(w.set_tool("Camera"));
        assert_eq!(w.active(), "Camera");
        assert!(w.set_tool("Onion Skin"));
        assert_eq!(w.active(), "Onion Skin");
        // Same canvas size throughout.
        assert_eq!(w.canvas().unwrap().width(), 64);
    }

    #[test]
    fn rejects_unknown() {
        let mut w = Workspace::new(16, 16);
        assert!(!w.set_tool("NotATool"));
        assert_eq!(w.active(), "Move");
    }

    #[test]
    fn differential() {
        let mut a = Workspace::new(32, 32);
        let mut b = Workspace::new(32, 32);
        a.set_tool("Lasso");
        b.set_tool("Pen");
        assert_ne!(a.line(), b.line());
        // Same canvas dimensions.
        assert_eq!(a.canvas().unwrap().width(), b.canvas().unwrap().width());
    }
}
