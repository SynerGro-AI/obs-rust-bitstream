//! Safe pure ShellLoop (sequence of shell actions).
//!
//! Runs a sequence: switch tool, set option, add layer,
//! edit. Returns the final status. No unwraps.
//!
//! Differential: a different sequence produces a different status.

use crate::active_tool::ToolSource;
use crate::edit_target::{EditApply, EditTarget};
use crate::history::History;
use crate::layer_group::GroupSet;
use crate::layer_select::LayerSelect;
use crate::layer_stack::LayerStack;
use crate::menu_history::MenuHistory;
use crate::ps_menus::PsMenu;
use crate::workspace::Workspace;

#[derive(Clone, Debug, PartialEq)]
pub enum ShellAction {
    Tool(&'static str),
    Size(u32),
    NewLayer,
    Opacity(f32),
}

pub struct ShellLoop;

impl ShellLoop {
    /// Run the sequence and return a status line.
    pub fn run(actions: &[ShellAction]) -> String {
        let mut workspace = Workspace::new(64, 64);
        let mut stack = LayerStack::new();
        let mut history = History::new();
        let groups = GroupSet::new();
        let mut select = LayerSelect::new();
        for action in actions {
            match action {
                ShellAction::Tool(name) => {
                    workspace.click(name, ToolSource::Photoshop);
                }
                ShellAction::Size(v) => {
                    workspace.options_mut().set("size", *v as f32);
                }
                ShellAction::NewLayer => {
                    MenuHistory::click(&mut stack, &mut history, PsMenu::Layer, "New Layer");
                    select.select_top(&stack);
                }
                ShellAction::Opacity(v) => {
                    if let Some(name) = select.selected() {
                        let _ = EditApply::set_opacity(
                            &mut stack,
                            &groups,
                            &EditTarget::Layer(name.to_string()),
                            *v,
                        );
                    }
                }
            }
        }
        let size = workspace.options().get("size").unwrap_or(0.0);
        format!(
            "tool={} size={:.0} layers={} selected={}",
            workspace.active(),
            size,
            stack.len(),
            select.selected().unwrap_or("none")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence() {
        let actions = [
            ShellAction::Tool("Brush"),
            ShellAction::Size(30),
            ShellAction::NewLayer,
            ShellAction::Opacity(0.5),
        ];
        let status = ShellLoop::run(&actions);
        assert!(status.contains("tool=Brush"));
        assert!(status.contains("size=30"));
        assert!(status.contains("layers=1"));
        assert!(status.contains("selected=Layer 1"));
    }

    #[test]
    fn empty() {
        let status = ShellLoop::run(&[]);
        assert!(status.contains("tool=Move"));
        assert!(status.contains("layers=0"));
    }

    #[test]
    fn differential() {
        let a = ShellLoop::run(&[ShellAction::NewLayer]);
        let b = ShellLoop::run(&[ShellAction::NewLayer, ShellAction::NewLayer]);
        assert_ne!(a, b);
    }
}
