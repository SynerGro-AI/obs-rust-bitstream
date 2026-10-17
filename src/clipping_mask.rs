//! Safe pure ClippingMask (layer clips to base core).
//!
//! A clipping mask makes one layer visible only where
//! the base layer is selected. Empty names rejected.
//! No unwraps.
//!
//! Differential: a clipped layer is limited to the base.

use crate::pixel_selection::Selection;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct ClippingMask {
    clips: BTreeMap<String, String>,
}

impl ClippingMask {
    pub fn new() -> Self {
        Self::default()
    }

    /// `layer` is clipped to `base`.
    pub fn clip(&mut self, layer: &str, base: &str) -> Result<(), ClipError> {
        if layer.is_empty() || base.is_empty() {
            return Err(ClipError::EmptyName);
        }
        self.clips.insert(layer.to_string(), base.to_string());
        Ok(())
    }

    pub fn base_of(&self, layer: &str) -> Result<&str, ClipError> {
        self.clips.get(layer).map(|s| s.as_str()).ok_or(ClipError::Missing)
    }

    pub fn unclip(&mut self, layer: &str) -> bool {
        self.clips.remove(layer).is_some()
    }

    /// A pixel is visible if the base selection has it.
    pub fn visible(&self, layer: &str, base_sel: &Selection, x: u32, y: u32) -> bool {
        if self.clips.get(layer).is_some() {
            base_sel.is_selected(x, y)
        } else {
            true
        }
    }

    pub fn len(&self) -> usize {
        self.clips.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut c = ClippingMask::new();
        assert!(c.clip("", "base").is_err());
        assert!(c.base_of("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut c = ClippingMask::new();
        c.clip("fg", "bg").unwrap();
        assert_eq!(c.base_of("fg").unwrap(), "bg");
        let mut base = Selection::new(4, 4);
        base.rect(1, 1, 2, 2).unwrap();
        assert!(c.visible("fg", &base, 1, 1));
        assert!(!c.visible("fg", &base, 0, 0));
        assert!(c.unclip("fg"));
    }
}
