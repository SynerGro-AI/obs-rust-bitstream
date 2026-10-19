//! Safe pure Workspace (one canvas, all tools).
//!
//! Every tool from Photoshop, Maya, AE, and ToonBoom
//! works on the same canvas. No workspace switch.
//! The active tool is just a name. No unwraps.
//!
//! Differential: a different active tool changes the name.

use crate::active_tool::{ActiveTool, ToolSource};
use crate::ae_tools::AeTools;
use crate::maya_shelves::MayaShelves;
use crate::pixel_canvas::PixelCanvas;
use crate::ps_toolbar::PsToolbar;
use crate::toon_tools::ToonTools;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    active: ActiveTool,
    width: u32,
    height: u32,
}

impl Workspace {
    /// One canvas. All tools available.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            active: ActiveTool::new(),
            width,
            height,
        }
    }

    pub fn active(&self) -> &str {
        self.active.name()
    }

    pub fn active_source(&self) -> ToolSource {
        self.active.source()
    }

    /// Toolbar click. Switches the active tool. Same canvas.
    pub fn click(&mut self, name: &str, source: ToolSource) -> bool {
        if !Self::all_tools().iter().any(|t| t == name) {
            return false;
        }
        self.active.activate(name, source)
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
        format!("tool={} source={} canvas={}x{}", self.active.name(), self.active.source().label(), self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_canvas() {
        let mut w = Workspace::new(64, 64);
        assert!(w.click("Brush", ToolSource::Photoshop));
        assert_eq!(w.active(), "Brush");
        assert!(w.click("Camera", ToolSource::Maya));
        assert_eq!(w.active(), "Camera");
        assert_eq!(w.active_source(), ToolSource::Maya);
        assert!(w.click("Onion Skin", ToolSource::ToonBoom));
        assert_eq!(w.active(), "Onion Skin");
        // Same canvas size throughout.
        assert_eq!(w.canvas().unwrap().width(), 64);
    }

    #[test]
    fn rejects_unknown() {
        let mut w = Workspace::new(16, 16);
        assert!(!w.click("NotATool", ToolSource::Photoshop));
        assert_eq!(w.active(), "Move");
    }

    #[test]
    fn differential() {
        let mut a = Workspace::new(32, 32);
        let mut b = Workspace::new(32, 32);
        a.click("Lasso", ToolSource::Photoshop);
        b.click("Pen", ToolSource::AfterEffects);
        assert_ne!(a.line(), b.line());
        // Same canvas dimensions.
        assert_eq!(a.canvas().unwrap().width(), b.canvas().unwrap().width());
    }
}
