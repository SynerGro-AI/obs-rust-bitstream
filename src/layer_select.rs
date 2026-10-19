//! Safe pure LayerSelect (choose the selected layer).
//!
//! Holds the selected layer name. select switches it.
//! Missing names are rejected. No unwraps.
//!
//! Differential: a different selection produces a different name.

use crate::layer_stack::LayerStack;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerSelect {
    selected: Option<String>,
}

impl LayerSelect {
    pub fn new() -> Self {
        Self { selected: None }
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Select a layer that exists. Returns false if missing.
    pub fn select(&mut self, stack: &LayerStack, name: &str) -> bool {
        if stack.layers().iter().any(|l| l.name() == name) {
            self.selected = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Select the top layer if any.
    pub fn select_top(&mut self, stack: &LayerStack) {
        self.selected = stack.layers().last().map(|l| l.name().to_string());
    }
}

impl Default for LayerSelect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        s.add("B").unwrap();
        let mut sel = LayerSelect::new();
        assert!(sel.select(&s, "A"));
        assert_eq!(sel.selected(), Some("A"));
        assert!(sel.select(&s, "B"));
        assert_eq!(sel.selected(), Some("B"));
    }

    #[test]
    fn missing_rejected() {
        let s = LayerStack::new();
        let mut sel = LayerSelect::new();
        assert!(!sel.select(&s, "Nope"));
        assert!(sel.selected().is_none());
    }

    #[test]
    fn differential() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        s.add("B").unwrap();
        let mut a = LayerSelect::new();
        let mut b = LayerSelect::new();
        a.select(&s, "A");
        b.select(&s, "B");
        assert_ne!(a.selected(), b.selected());
    }
}
