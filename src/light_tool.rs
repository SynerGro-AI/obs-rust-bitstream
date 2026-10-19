//! Safe pure LightTool (light placement tool).
//!
//! Holds named lights with position, intensity, and color.
//! Add, set intensity, or remove. Empty names rejected.
//! No unwraps.
//!
//! Differential: a different intensity produces a different light.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, Default)]
pub struct LightTool {
    lights: HashMap<String, Light>,
}

impl LightTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a light. False if empty or duplicate.
    pub fn add(&mut self, name: &str, x: f32, y: f32, z: f32, intensity: f32) -> bool {
        if name.is_empty() || self.lights.contains_key(name) {
            return false;
        }
        self.lights.insert(
            name.to_string(),
            Light {
                x,
                y,
                z,
                intensity: intensity.max(0.0),
                r: 255,
                g: 255,
                b: 255,
            },
        );
        true
    }

    pub fn set_intensity(&mut self, name: &str, intensity: f32) -> bool {
        if let Some(l) = self.lights.get_mut(name) {
            l.intensity = intensity.max(0.0);
            true
        } else {
            false
        }
    }

    pub fn set_color(&mut self, name: &str, r: u8, g: u8, b: u8) -> bool {
        if let Some(l) = self.lights.get_mut(name) {
            l.r = r;
            l.g = g;
            l.b = b;
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<&Light> {
        self.lights.get(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.lights.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.lights.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_intensity() {
        let mut l = LightTool::new();
        assert!(l.add("key", 0.0, 2.0, 0.0, 1.0));
        assert!(l.set_intensity("key", 2.5));
        assert!((l.get("key").unwrap().intensity - 2.5).abs() < 0.01);
    }

    #[test]
    fn empty_rejected() {
        let mut l = LightTool::new();
        assert!(!l.add("", 0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn differential() {
        let mut a = LightTool::new();
        a.add("l", 0.0, 0.0, 0.0, 1.0);
        let mut b = LightTool::new();
        b.add("l", 0.0, 0.0, 0.0, 3.0);
        assert_ne!(a.get("l").unwrap().intensity, b.get("l").unwrap().intensity);
    }
}
