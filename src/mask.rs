//! Safe pure Mask (layer mask core).
//!
//! A mask owns a name and an invert flag. It can be attached
//! to a layer by name. Empty names rejected. No unwraps.
//!
//! Differential: attach + invert round-trips.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaskError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Mask {
    name: String,
    inverted: bool,
    feather: f32,
    density: f32,
}

impl Mask {
    pub fn new(name: &str) -> Result<Self, MaskError> {
        if name.is_empty() {
            return Err(MaskError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            inverted: false,
            feather: 0.0,
            density: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_inverted(&mut self, v: bool) {
        self.inverted = v;
    }

    pub fn inverted(&self) -> bool {
        self.inverted
    }

    pub fn set_feather(&mut self, f: f32) {
        self.feather = f.max(0.0);
    }

    pub fn feather(&self) -> f32 {
        self.feather
    }

    pub fn set_density(&mut self, d: f32) {
        self.density = d.clamp(0.0, 1.0);
    }

    pub fn density(&self) -> f32 {
        self.density
    }

    /// Effective opacity of the mask at a sample.
    pub fn apply(&self, sample: f32) -> f32 {
        let s = if self.inverted { 1.0 - sample } else { sample };
        (s * self.density).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaskSet {
    masks: HashMap<String, Mask>,
    attached: HashMap<String, String>,
}

impl MaskSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), MaskError> {
        if name.is_empty() {
            return Err(MaskError::EmptyName);
        }
        if self.masks.contains_key(name) {
            return Err(MaskError::Duplicate);
        }
        self.masks.insert(name.to_string(), Mask::new(name)?);
        Ok(())
    }

    /// Attach a mask to a layer.
    pub fn attach(&mut self, layer: &str, mask: &str) -> Result<(), MaskError> {
        if !self.masks.contains_key(mask) {
            return Err(MaskError::Missing);
        }
        self.attached.insert(layer.to_string(), mask.to_string());
        Ok(())
    }

    pub fn mask_of(&self, layer: &str) -> Option<&Mask> {
        self.attached
            .get(layer)
            .and_then(|n| self.masks.get(n))
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Mask, MaskError> {
        self.masks.get_mut(name).ok_or(MaskError::Missing)
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach() {
        let mut s = MaskSet::new();
        s.add("soft").unwrap();
        s.attach("fg", "soft").unwrap();
        assert!(s.mask_of("fg").is_some());
    }

    #[test]
    fn empty_name() {
        let mut s = MaskSet::new();
        assert!(s.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = MaskSet::new();
        s.add("hard").unwrap();
        s.get_mut("hard").unwrap().set_inverted(true);
        s.attach("bg", "hard").unwrap();
        assert!(s.mask_of("bg").unwrap().inverted());
    }

    #[test]
    fn feather_and_density() {
        let mut s = MaskSet::new();
        s.add("soft").unwrap();
        s.get_mut("soft").unwrap().set_feather(4.0);
        s.get_mut("soft").unwrap().set_density(0.5);
        assert_eq!(s.get_mut("soft").unwrap().feather(), 4.0);
        assert!((s.get_mut("soft").unwrap().apply(1.0) - 0.5).abs() < 0.001);
    }
}
