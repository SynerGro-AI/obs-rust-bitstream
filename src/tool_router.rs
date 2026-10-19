//! Safe pure ToolRouter (input → active tool).
//!
//! Routes a key or click to the active tool.
//! Hotkeys from the toolbars switch the tool.
//! Other keys and clicks are reported as actions
//! on the current tool. No unwraps.
//!
//! Differential: a different tool produces a different action.

use crate::active_tool::{ActiveTool, ToolSource};
use crate::input_device::InputDevice;
use crate::ps_toolbar::PsToolbar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolAction {
    /// A hotkey switched the active tool.
    Switched { to: String },
    /// A key was pressed while a tool was active.
    Key { tool: String, key: String },
    /// A click landed while a tool was active.
    Click { tool: String, x: i32, y: i32 },
    /// Nothing to route.
    None,
}

pub struct ToolRouter;

impl ToolRouter {
    /// Route the last input to the active tool.
    /// Hotkeys switch the tool. Other input becomes an action.
    pub fn route(active: &mut ActiveTool, input: &InputDevice) -> ToolAction {
        if let Some(key) = input.last_key() {
            if let Some(name) = Self::hotkey_tool(key) {
                active.activate(name, ToolSource::Photoshop);
                return ToolAction::Switched { to: name.to_string() };
            }
            return ToolAction::Key {
                tool: active.name().to_string(),
                key: key.to_string(),
            };
        }
        if let Some((x, y)) = input.last_click() {
            return ToolAction::Click {
                tool: active.name().to_string(),
                x: x as i32,
                y: y as i32,
            };
        }
        ToolAction::None
    }

    /// Photoshop hotkey → tool name.
    fn hotkey_tool(key: &str) -> Option<&'static str> {
        PsToolbar::tools()
            .into_iter()
            .find(|t| t.hotkey.eq_ignore_ascii_case(key))
            .map(|t| t.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_switches() {
        let mut active = ActiveTool::new();
        let mut input = InputDevice::new();
        input.press("B");
        match ToolRouter::route(&mut active, &input) {
            ToolAction::Switched { to } => assert_eq!(to, "Brush"),
            _ => panic!("expected switch"),
        }
        assert_eq!(active.name(), "Brush");
    }

    #[test]
    fn click_routes_to_tool() {
        let mut active = ActiveTool::new();
        active.activate("Brush", ToolSource::Photoshop);
        let mut input = InputDevice::new();
        input.click(10.0, 20.0);
        match ToolRouter::route(&mut active, &input) {
            ToolAction::Click { tool, x, y } => {
                assert_eq!(tool, "Brush");
                assert_eq!((x, y), (10, 20));
            }
            _ => panic!("expected click"),
        }
    }

    #[test]
    fn differential() {
        let mut a = ActiveTool::new();
        let mut b = ActiveTool::new();
        a.activate("Brush", ToolSource::Photoshop);
        b.activate("Lasso", ToolSource::Photoshop);
        let mut input = InputDevice::new();
        input.click(1.0, 1.0);
        let aa = ToolRouter::route(&mut a, &input);
        let bb = ToolRouter::route(&mut b, &input);
        assert_ne!(aa, bb);
    }
}
