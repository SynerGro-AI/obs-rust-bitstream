//! Safe pure ShellEvent (input → dispatch → state).
//!
//! Reads a key or click and calls the existing dispatch.
//! Hotkeys switch the tool. Menu keys add a layer.
//! Clicks are recorded. No unwraps.
//!
//! Differential: a different key produces a different state.

use crate::active_tool::{ActiveTool, ToolSource};
use crate::history::History;
use crate::input_device::InputDevice;
use crate::layer_stack::LayerStack;
use crate::menu_history::MenuHistory;
use crate::ps_menus::PsMenu;
use crate::tool_router::{ToolAction, ToolRouter};
use crate::workspace::Workspace;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellInput {
    Key(&'static str),
    Click(i32, i32),
}

pub struct ShellEvent;

impl ShellEvent {
    /// Handle one input. Returns a one-line result.
    pub fn handle(
        workspace: &mut Workspace,
        stack: &mut LayerStack,
        history: &mut History,
        input: ShellInput,
    ) -> String {
        match input {
            ShellInput::Key(k) => {
                // Menu shortcuts.
                if k == "n" {
                    MenuHistory::click(stack, history, PsMenu::Layer, "New Layer");
                    return format!("action=new_layer layers={}", stack.len());
                }
                if k == "z" {
                    MenuHistory::click(stack, history, PsMenu::Edit, "Undo");
                    return format!("action=undo layers={}", stack.len());
                }
                // Tool hotkey.
                let mut device = InputDevice::new();
                device.press(k);
                let mut active = ActiveTool::new();
                active.activate(workspace.active(), workspace.active_source());
                let action = ToolRouter::route(&mut active, &device);
                if let ToolAction::Switched { to } = &action {
                    workspace.click(to, ToolSource::Photoshop);
                }
                format!("action={:?} tool={}", action, workspace.active())
            }
            ShellInput::Click(x, y) => {
                format!("action=click x={} y={} tool={}", x, y, workspace.active())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_switches() {
        let mut w = Workspace::new(32, 32);
        let mut s = LayerStack::new();
        let mut h = History::new();
        let result = ShellEvent::handle(&mut w, &mut s, &mut h, ShellInput::Key("B"));
        assert!(result.contains("Brush"));
        assert_eq!(w.active(), "Brush");
    }

    #[test]
    fn new_layer() {
        let mut w = Workspace::new(32, 32);
        let mut s = LayerStack::new();
        let mut h = History::new();
        let result = ShellEvent::handle(&mut w, &mut s, &mut h, ShellInput::Key("n"));
        assert!(result.contains("new_layer"));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn click_recorded() {
        let mut w = Workspace::new(32, 32);
        let mut s = LayerStack::new();
        let mut h = History::new();
        let result = ShellEvent::handle(&mut w, &mut s, &mut h, ShellInput::Click(10, 20));
        assert!(result.contains("click"));
        assert!(result.contains("x=10"));
    }

    #[test]
    fn differential() {
        let mut w = Workspace::new(32, 32);
        let mut s = LayerStack::new();
        let mut h = History::new();
        let a = ShellEvent::handle(&mut w, &mut s, &mut h, ShellInput::Key("B"));
        let b = ShellEvent::handle(&mut w, &mut s, &mut h, ShellInput::Key("L"));
        assert_ne!(a, b);
    }
}
