//! Safe pure MenuHistory (menu actions record and undo).
//!
//! New Layer pushes a step. Undo pops it.
//! Redo restores it. No unwraps.
//!
//! Differential: undo after a step reduces the count.

use crate::history::History;
use crate::layer_stack::LayerStack;
use crate::menu_path::MenuPath;
use crate::ps_menus::PsMenu;

pub struct MenuHistory;

impl MenuHistory {
    /// Click and record. Undo/Redo use the history.
    /// Returns true if the stack or history changed.
    pub fn click(stack: &mut LayerStack, history: &mut History, menu: PsMenu, item: &str) -> bool {
        match (menu, item) {
            (PsMenu::Edit, "Undo") => {
                if let Some(name) = history.undo() {
                    // Reverse a layer add if that is what was recorded.
                    let _ = stack.remove(&name);
                    true
                } else {
                    false
                }
            }
            (PsMenu::Edit, "Redo") => {
                if let Some(name) = history.redo() {
                    // Restore a layer add if that is what was undone.
                    let _ = stack.add(&name);
                    true
                } else {
                    false
                }
            }
            _ => {
                let before = stack.len();
                if MenuPath::click(stack, menu, item) {
                    // Record the layer that was added, if any.
                    if stack.len() > before {
                        if let Some(last) = stack.layers().last() {
                            history.push(last.name());
                        }
                    } else {
                        history.push(item);
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_undoes() {
        let mut s = LayerStack::new();
        let mut h = History::new();
        assert!(MenuHistory::click(
            &mut s,
            &mut h,
            PsMenu::Layer,
            "New Layer"
        ));
        assert_eq!(s.len(), 1);
        assert!(h.can_undo());
        assert!(MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Undo"));
        assert_eq!(s.len(), 0);
        assert!(!h.can_undo());
    }

    #[test]
    fn redo_restores() {
        let mut s = LayerStack::new();
        let mut h = History::new();
        MenuHistory::click(&mut s, &mut h, PsMenu::Layer, "New Layer");
        MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Undo");
        assert_eq!(s.len(), 0);
        assert!(MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Redo"));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn undo_empty_is_false() {
        let mut s = LayerStack::new();
        let mut h = History::new();
        assert!(!MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Undo"));
    }

    #[test]
    fn differential() {
        let mut s = LayerStack::new();
        let mut h = History::new();
        MenuHistory::click(&mut s, &mut h, PsMenu::Layer, "New Layer");
        let before = h.len();
        MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Undo");
        assert!(h.len() < before || !h.can_undo());
    }
}
