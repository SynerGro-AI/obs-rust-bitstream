//! Safe pure Light (scene light core).
//!
//! A light owns a name, a position, a color, and an intensity.
//! Empty names are rejected. No unwraps.
//!
//! Differential: set intensity + get round-trips.

use crate::vec3::Vec3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LightError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Light {
    name: String,
    position: Vec3,
    color: Vec3,
    intensity: f32,
}

impl Light {
    pub fn new(name: &str) -> Result<Self, LightError> {
        if name.is_empty() {
            return Err(LightError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            position: Vec3::new(0.0, 10.0, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_position(&mut self, p: Vec3) {
        self.position = p;
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_color(&mut self, c: Vec3) {
        self.color = c;
    }

    pub fn color(&self) -> Vec3 {
        self.color
    }

    pub fn set_intensity(&mut self, i: f32) {
        self.intensity = i.max(0.0);
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let l = Light::new("sun").unwrap();
        assert_eq!(l.name(), "sun");
        assert_eq!(l.intensity(), 1.0);
    }

    #[test]
    fn empty_name() {
        assert!(Light::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut l = Light::new("key").unwrap();
        l.set_intensity(2.5);
        l.set_position(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(l.intensity(), 2.5);
        assert_eq!(l.position().y, 2.0);
    }
}
