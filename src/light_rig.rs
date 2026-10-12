//! Safe pure LightRig (multi-light core).
//!
//! Holds more than one light. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add two lights + count round-trips.

use std::collections::HashMap;

use crate::light::{Light, LightError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LightRigError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<LightError> for LightRigError {
    fn from(_: LightError) -> Self {
        LightRigError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct LightRig {
    lights: HashMap<String, Light>,
}

impl LightRig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), LightRigError> {
        if name.is_empty() {
            return Err(LightRigError::EmptyName);
        }
        if self.lights.contains_key(name) {
            return Err(LightRigError::Duplicate);
        }
        let l = Light::new(name)?;
        self.lights.insert(name.to_string(), l);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Light, LightRigError> {
        self.lights.get(name).ok_or(LightRigError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Light, LightRigError> {
        self.lights.get_mut(name).ok_or(LightRigError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.lights.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.lights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lights.is_empty()
    }

    /// Total intensity of every light in the rig.
    pub fn total_intensity(&self) -> f32 {
        self.lights.values().map(|l| l.intensity()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut r = LightRig::new();
        r.add("key").unwrap();
        r.add("fill").unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = LightRig::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(LightRigError::Duplicate)));
        assert!(matches!(r.add(""), Err(LightRigError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut r = LightRig::new();
        r.add("key").unwrap();
        r.add("rim").unwrap();
        r.get_mut("key").unwrap().set_intensity(2.0);
        r.get_mut("rim").unwrap().set_intensity(0.5);
        assert!((r.total_intensity() - 2.5).abs() < 0.001);
    }
}
