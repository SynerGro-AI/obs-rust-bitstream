//! Safe pure LayerMask (per-layer mask core).
//!
//! Attaches a Selection as a mask to a named layer.
//! Empty names rejected. No unwraps.
//!
//! Differential: attach + query round-trips.

use crate::pixel_selection::Selection;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaskError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct LayerMask {
    masks: BTreeMap<String, Selection>,
}

impl LayerMask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attach(&mut self, layer: &str, mask: Selection) -> Result<(), MaskError> {
        if layer.is_empty() {
            return Err(MaskError::EmptyName);
        }
        self.masks.insert(layer.to_string(), mask);
        Ok(())
    }

    pub fn get(&self, layer: &str) -> Result<&Selection, MaskError> {
        self.masks.get(layer).ok_or(MaskError::Missing)
    }

    pub fn remove(&mut self, layer: &str) -> bool {
        self.masks.remove(layer).is_some()
    }

    pub fn has(&self, layer: &str) -> bool {
        self.masks.contains_key(layer)
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut m = LayerMask::new();
        let s = Selection::new(2, 2);
        assert!(m.attach("", s).is_err());
        assert!(m.get("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut m = LayerMask::new();
        let mut s = Selection::new(4, 4);
        s.rect(1, 1, 2, 2).unwrap();
        m.attach("fg", s).unwrap();
        assert!(m.has("fg"));
        assert!(m.get("fg").unwrap().is_selected(1, 1));
        assert!(m.remove("fg"));
        assert!(!m.has("fg"));
    }
}
