//! Safe pure PuppetPin (deform tool).
//!
//! Holds named pins with positions. Add a pin, move it,
//! or remove it. Empty names rejected. No unwraps.
//!
//! Differential: moving a pin changes its position.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Pin {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default)]
pub struct PuppetPin {
    pins: HashMap<String, Pin>,
}

impl PuppetPin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pin. False if the name is empty or already exists.
    pub fn add(&mut self, name: &str, x: f32, y: f32) -> bool {
        if name.is_empty() || self.pins.contains_key(name) {
            return false;
        }
        self.pins.insert(name.to_string(), Pin { x, y });
        true
    }

    /// Move a pin. False if missing.
    pub fn move_pin(&mut self, name: &str, x: f32, y: f32) -> bool {
        if let Some(p) = self.pins.get_mut(name) {
            p.x = x;
            p.y = y;
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<&Pin> {
        self.pins.get(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.pins.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.pins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_move() {
        let mut p = PuppetPin::new();
        assert!(p.add("arm", 0.0, 0.0));
        assert!(p.move_pin("arm", 10.0, 5.0));
        let pin = p.get("arm").unwrap();
        assert!((pin.x - 10.0).abs() < 0.01);
    }

    #[test]
    fn empty_rejected() {
        let mut p = PuppetPin::new();
        assert!(!p.add("", 0.0, 0.0));
    }

    #[test]
    fn differential() {
        let mut a = PuppetPin::new();
        a.add("p", 0.0, 0.0);
        let mut b = PuppetPin::new();
        b.add("p", 0.0, 0.0);
        b.move_pin("p", 5.0, 5.0);
        assert_ne!(a.get("p").unwrap().x, b.get("p").unwrap().x);
    }
}
