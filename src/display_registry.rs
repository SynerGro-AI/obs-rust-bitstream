//! Safe pure DisplayRegistry (OBS display lookup core).
//!
//! Name-to-DisplayContext map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + get returns the same display; size round-trips.

use std::collections::HashMap;

use crate::display_context::{DisplayContext, DisplayError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<DisplayError> for DisplayRegistryError {
    fn from(_: DisplayError) -> Self {
        DisplayRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct DisplayRegistry {
    displays: HashMap<String, DisplayContext>,
}

impl DisplayRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, width: u32, height: u32) -> Result<(), DisplayRegistryError> {
        if name.is_empty() {
            return Err(DisplayRegistryError::EmptyName);
        }
        if self.displays.contains_key(name) {
            return Err(DisplayRegistryError::Duplicate);
        }
        let ctx = DisplayContext::new(name, width, height)?;
        self.displays.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&DisplayContext, DisplayRegistryError> {
        self.displays.get(name).ok_or(DisplayRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut DisplayContext, DisplayRegistryError> {
        self.displays
            .get_mut(name)
            .ok_or(DisplayRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.displays.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.displays.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.displays.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = DisplayRegistry::new();
        r.add("preview", 1280, 720).unwrap();
        assert_eq!(r.get("preview").unwrap().size(), (1280, 720));
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = DisplayRegistry::new();
        r.add("a", 1, 1).unwrap();
        assert!(matches!(
            r.add("a", 1, 1),
            Err(DisplayRegistryError::Duplicate)
        ));
        assert!(matches!(
            r.add("", 1, 1),
            Err(DisplayRegistryError::EmptyName)
        ));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = DisplayRegistry::new();
        assert!(matches!(r.get("nope"), Err(DisplayRegistryError::Missing)));
        r.add("d", 1, 1).unwrap();
        assert!(r.remove("d"));
        assert!(!r.contains("d"));
    }

    #[test]
    fn differential() {
        let mut r = DisplayRegistry::new();
        r.add("program", 1920, 1080).unwrap();
        r.get_mut("program").unwrap().set_source("Main");
        assert_eq!(r.get("program").unwrap().source(), Some("Main"));
    }
}
