//! Safe pure UiView (full shell in one text).
//!
//! Shows the active tool, its options, and the layer
//! panel. One canvas, all tools. No unwraps.
//!
//! Differential: a different tool produces a different view.

use crate::layer_stack::LayerStack;
use crate::properties::Properties;
use crate::ps_layers::PsLayers;
use crate::workspace::Workspace;

pub struct UiView;

impl UiView {
    /// Multi-line view of the workspace and layer stack.
    pub fn render(workspace: &Workspace, stack: &LayerStack) -> String {
        let mut out = String::new();
        out.push_str(&format!("tool={}\n", workspace.active()));
        out.push_str(&format!("options={}\n", workspace.options().line()));
        out.push_str("layers:\n");
        let panel = PsLayers::panel(stack);
        if panel.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for line in panel {
                out.push_str(&format!("  {}\n", line));
            }
            // Selected is the top layer.
            if let Some(top) = stack.layers().last() {
                if let Some(props) = Properties::of(stack, top.name()) {
                    out.push_str(&format!("selected: {}\n", Properties::line(&props)));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_tool::ToolSource;

    #[test]
    fn shows_tool_and_layers() {
        let mut w = Workspace::new(32, 32);
        w.click("Brush", ToolSource::Photoshop);
        let mut s = LayerStack::new();
        s.add("Layer 1").unwrap();
        let view = UiView::render(&w, &s);
        assert!(view.contains("tool=Brush"));
        assert!(view.contains("size="));
        assert!(view.contains("Layer 1"));
        assert!(view.contains("selected:"));
    }

    #[test]
    fn empty_layers() {
        let w = Workspace::new(16, 16);
        let s = LayerStack::new();
        let view = UiView::render(&w, &s);
        assert!(view.contains("(none)"));
    }

    #[test]
    fn differential() {
        let mut a = Workspace::new(16, 16);
        let mut b = Workspace::new(16, 16);
        a.click("Brush", ToolSource::Photoshop);
        b.click("Crop", ToolSource::Photoshop);
        let s = LayerStack::new();
        assert_ne!(UiView::render(&a, &s), UiView::render(&b, &s));
    }
}
