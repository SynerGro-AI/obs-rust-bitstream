//! Safe pure MenuPath (click → dispatch → apply).
//!
//! One call: menu + item → action → stack change.
//! Unknown items do nothing. No unwraps.
//!
//! Differential: New Layer increases the count;
//! an unknown item does not.

use crate::layer_stack::LayerStack;
use crate::menu_apply::MenuApply;
use crate::menu_dispatch::MenuDispatch;
use crate::ps_menus::PsMenu;

pub struct MenuPath;

impl MenuPath {
    /// Click a menu item and apply it. Returns true if the stack changed.
    pub fn click(stack: &mut LayerStack, menu: PsMenu, item: &str) -> bool {
        match MenuDispatch::click(menu, item) {
            Some(action) => MenuApply::apply(stack, &action),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_adds_layer() {
        let mut s = LayerStack::new();
        assert!(MenuPath::click(&mut s, PsMenu::Layer, "New Layer"));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn unknown_does_nothing() {
        let mut s = LayerStack::new();
        assert!(!MenuPath::click(&mut s, PsMenu::Layer, "NotAThing"));
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn differential() {
        let mut a = LayerStack::new();
        let mut b = LayerStack::new();
        MenuPath::click(&mut a, PsMenu::Layer, "New Layer");
        MenuPath::click(&mut b, PsMenu::Layer, "New Layer");
        MenuPath::click(&mut b, PsMenu::Layer, "New Layer");
        assert!(b.len() > a.len());
    }
}
