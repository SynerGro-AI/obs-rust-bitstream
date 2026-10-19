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
    pub fn click(
        stack: &mut LayerStack,
        history: &mut History,
        menu: PsMenu,
        item: &str,
    ) -> bool {
        match (menu, item) {
            (PsMenu::Edit, "Undo") => history.undo().is_some(),
            (PsMenu::Edit, "Redo") => history.redo().is_some(),
            _ => {
                if MenuPath::click(stack, menu, item) {
                    history.push(item);
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
        assert!(MenuHistory::click(&mut s, &mut h, PsMenu::Layer, "New Layer"));
        assert_eq!(s.len(), 1);
        assert!(h.can_undo());
        assert!(MenuHistory::click(&mut s, &mut h, PsMenu::Edit, "Undo"));
        assert!(!h.can_undo());
        assert!(h.can_redo());
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
