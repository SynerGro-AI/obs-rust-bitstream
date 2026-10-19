//! Safe pure UiView (full shell in one text).
//!
//! Shows the active tool, its options, and the layer
//! panel. One canvas, all tools. No unwraps.
//!
//! Differential: a different tool produces a different view.

use crate::edit_target::EditTarget;
use crate::layer_select::LayerSelect;
use crate::layer_stack::LayerStack;
use crate::properties::Properties;
use crate::ps_layers::PsLayers;
use crate::tool_help::ToolHelp;
use crate::workspace::Workspace;

pub struct UiView;

impl UiView {
    /// Multi-line view of the workspace, layer stack, and selection.
    pub fn render(workspace: &Workspace, stack: &LayerStack, select: &LayerSelect) -> String {
        let mut out = String::new();
        out.push_str(&format!("tool={}\n", workspace.active()));
        out.push_str(&format!("thought: {}\n", ToolHelp::tool(workspace.active())));
        out.push_str(&format!("options={}\n", workspace.options().line()));
        out.push_str("layers:\n");
        let panel = PsLayers::panel(stack);
        if panel.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for line in panel {
                out.push_str(&format!("  {}\n", line));
            }
            // Selected layer, or the top if none chosen.
            let name = select
                .selected()
                .map(|s| s.to_string())
                .or_else(|| stack.layers().last().map(|l| l.name().to_string()));
            if let Some(name) = name {
                if let Some(props) = Properties::of(stack, &name) {
                    out.push_str(&format!("selected: {}\n", Properties::line(&props)));
                }
            }
        }
        out
    }

    /// View plus the edit target.
    pub fn render_with_target(
        workspace: &Workspace,
        stack: &LayerStack,
        select: &LayerSelect,
        target: &EditTarget,
    ) -> String {
        let mut out = Self::render(workspace, stack, select);
        let kind = match target {
            EditTarget::Layer(n) => format!("layer {}", n),
            EditTarget::Group(n) => format!("group {}", n),
        };
        out.push_str(&format!("editing: {}\n", kind));
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::edit_target::EditTarget;
    use crate::layer_select::LayerSelect;
    use super::*;
    use crate::active_tool::ToolSource;

    #[test]
    fn shows_tool_and_layers() {
        let mut w = Workspace::new(32, 32);
        w.click("Brush", ToolSource::Photoshop);
        let mut s = LayerStack::new();
        s.add("Layer 1").unwrap();
        let view = UiView::render(&w, &s, &LayerSelect::new());
        assert!(view.contains("tool=Brush"));
        assert!(view.contains("thought:"));
        assert!(view.contains("💭"));
        assert!(view.contains("size="));
        assert!(view.contains("Layer 1"));
        assert!(view.contains("selected:"));
    }

    #[test]
    fn empty_layers() {
        let w = Workspace::new(16, 16);
        let s = LayerStack::new();
        let view = UiView::render(&w, &s, &LayerSelect::new());
        assert!(view.contains("(none)"));
    }

    #[test]
    fn shows_edit_target() {
        let w = Workspace::new(16, 16);
        let s = LayerStack::new();
        let view = UiView::render_with_target(&w, &s, &LayerSelect::new(), &EditTarget::Layer("A".into()));
        assert!(view.contains("editing: layer A"));
        let view = UiView::render_with_target(&w, &s, &LayerSelect::new(), &EditTarget::Group("G".into()));
        assert!(view.contains("editing: group G"));
    }

    #[test]
    fn differential() {
        let mut a = Workspace::new(16, 16);
        let mut b = Workspace::new(16, 16);
        a.click("Brush", ToolSource::Photoshop);
        b.click("Crop", ToolSource::Photoshop);
        let s = LayerStack::new();
        assert_ne!(UiView::render(&a, &s, &LayerSelect::new()), UiView::render(&b, &s, &LayerSelect::new()));
    }
}
