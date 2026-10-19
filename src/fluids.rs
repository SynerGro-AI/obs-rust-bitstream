//! Safe pure Fluids (emitter tool).
//!
//! Holds named emitters with position and rate.
//! Add, set rate, or remove. Empty names rejected.
//! No unwraps.
//!
//! Differential: a different rate produces a different emitter.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Emitter {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rate: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Fluids {
    emitters: HashMap<String, Emitter>,
}

impl Fluids {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an emitter. False if empty or duplicate.
    pub fn add(&mut self, name: &str, x: f32, y: f32, z: f32, rate: f32) -> bool {
        if name.is_empty() || self.emitters.contains_key(name) {
            return false;
        }
        self.emitters.insert(
            name.to_string(),
            Emitter { x, y, z, rate: rate.max(0.0) },
        );
        true
    }

    pub fn set_rate(&mut self, name: &str, rate: f32) -> bool {
        if let Some(e) = self.emitters.get_mut(name) {
            e.rate = rate.max(0.0);
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<&Emitter> {
        self.emitters.get(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.emitters.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.emitters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_rate() {
        let mut f = Fluids::new();
        assert!(f.add("smoke", 0.0, 1.0, 0.0, 10.0));
        assert!(f.set_rate("smoke", 20.0));
        assert!((f.get("smoke").unwrap().rate - 20.0).abs() < 0.01);
    }

    #[test]
    fn empty_rejected() {
        let mut f = Fluids::new();
        assert!(!f.add("", 0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn differential() {
        let mut a = Fluids::new();
        a.add("e", 0.0, 0.0, 0.0, 1.0);
        let mut b = Fluids::new();
        b.add("e", 0.0, 0.0, 0.0, 5.0);
        assert_ne!(a.get("e").unwrap().rate, b.get("e").unwrap().rate);
    }
}
