//! Safe pure MenuApply (action → layer stack).
//!
//! Applies a MenuAction to a LayerStack.
//! New Layer adds a layer. Unknown or no-op
//! actions leave the stack unchanged. No unwraps.
//!
//! Differential: New Layer increases the count.

use crate::layer_stack::LayerStack;
use crate::menu_dispatch::MenuAction;

pub struct MenuApply;

impl MenuApply {
    /// Apply the action. Returns true if the stack changed.
    pub fn apply(stack: &mut LayerStack, action: &MenuAction) -> bool {
        match action {
            MenuAction::NewLayer => {
                let name = format!("Layer {}", stack.len() + 1);
                stack.add(&name).is_ok()
            }
            MenuAction::DuplicateLayer => {
                if stack.len() == 0 {
                    return false;
                }
                let name = format!("Layer {} copy", stack.len());
                stack.add(&name).is_ok()
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer_adds() {
        let mut s = LayerStack::new();
        assert!(MenuApply::apply(&mut s, &MenuAction::NewLayer));
        assert_eq!(s.len(), 1);
        assert!(MenuApply::apply(&mut s, &MenuAction::NewLayer));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn undo_is_noop() {
        let mut s = LayerStack::new();
        assert!(!MenuApply::apply(&mut s, &MenuAction::Undo));
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn differential() {
        let mut a = LayerStack::new();
        let mut b = LayerStack::new();
        MenuApply::apply(&mut a, &MenuAction::NewLayer);
        MenuApply::apply(&mut b, &MenuAction::NewLayer);
        MenuApply::apply(&mut b, &MenuAction::NewLayer);
        assert!(b.len() > a.len());
    }
}
