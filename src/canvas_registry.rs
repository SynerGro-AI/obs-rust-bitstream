//! Safe pure CanvasRegistry (OBS canvas lookup core).
//!
//! Name-to-Canvas map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + set scene through the registry round-trips.

use std::collections::HashMap;

use crate::canvas::{Canvas, CanvasError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<CanvasError> for CanvasRegistryError {
    fn from(_: CanvasError) -> Self {
        CanvasRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct CanvasRegistry {
    canvases: HashMap<String, Canvas>,
}

impl CanvasRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, width: u32, height: u32) -> Result<(), CanvasRegistryError> {
        if name.is_empty() {
            return Err(CanvasRegistryError::EmptyName);
        }
        if self.canvases.contains_key(name) {
            return Err(CanvasRegistryError::Duplicate);
        }
        let c = Canvas::new(name, width, height)?;
        self.canvases.insert(name.to_string(), c);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Canvas, CanvasRegistryError> {
        self.canvases.get(name).ok_or(CanvasRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Canvas, CanvasRegistryError> {
        self.canvases
            .get_mut(name)
            .ok_or(CanvasRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.canvases.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.canvases.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.canvases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = CanvasRegistry::new();
        r.add("preview", 1280, 720).unwrap();
        assert_eq!(r.get("preview").unwrap().size(), (1280, 720));
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = CanvasRegistry::new();
        r.add("a", 1, 1).unwrap();
        assert!(matches!(
            r.add("a", 1, 1),
            Err(CanvasRegistryError::Duplicate)
        ));
        assert!(matches!(
            r.add("", 1, 1),
            Err(CanvasRegistryError::EmptyName)
        ));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = CanvasRegistry::new();
        assert!(matches!(r.get("nope"), Err(CanvasRegistryError::Missing)));
        r.add("c", 1, 1).unwrap();
        assert!(r.remove("c"));
        assert!(!r.contains("c"));
    }

    #[test]
    fn differential() {
        let mut r = CanvasRegistry::new();
        r.add("program", 1920, 1080).unwrap();
        r.get_mut("program").unwrap().set_scene("Main");
        assert_eq!(r.get("program").unwrap().scene(), Some("Main"));
    }
}
