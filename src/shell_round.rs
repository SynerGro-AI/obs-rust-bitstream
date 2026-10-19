//! Safe pure ShellRound (tool + option + layer + view).
//!
//! Confirms the full shell path: switch tool, set an
//! option, add a layer, render the view. No unwraps.
//!
//! Differential: a different tool produces a different view.

use crate::active_tool::ToolSource;
use crate::history::History;
use crate::layer_stack::LayerStack;
use crate::menu_history::MenuHistory;
use crate::ps_menus::PsMenu;
use crate::ui_view::UiView;
use crate::workspace::Workspace;

pub struct ShellRound;

impl ShellRound {
    /// Run the path and return the view.
    pub fn run() -> String {
        let mut workspace = Workspace::new(64, 64);
        workspace.click("Brush", ToolSource::Photoshop);
        workspace.options_mut().set("size", 30.0);
        let mut stack = LayerStack::new();
        let mut history = History::new();
        MenuHistory::click(&mut stack, &mut history, PsMenu::Layer, "New Layer");
        UiView::render(&workspace, &stack)
    }

    /// True if the view contains the tool, the option, and the layer.
    pub fn complete(view: &str) -> bool {
        view.contains("tool=Brush") && view.contains("size=30") && view.contains("Layer 1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_path() {
        let view = ShellRound::run();
        assert!(ShellRound::complete(&view));
    }

    #[test]
    fn view_has_sections() {
        let view = ShellRound::run();
        assert!(view.contains("options="));
        assert!(view.contains("layers:"));
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
