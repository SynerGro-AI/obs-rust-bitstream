//! Safe pure HotkeyRegistry (OBS hotkey lookup core).
//!
//! Name-to-Hotkey map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + press through the registry round-trips.

use std::collections::HashMap;

use crate::hotkey::{Hotkey, HotkeyError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<HotkeyError> for HotkeyRegistryError {
    fn from(_: HotkeyError) -> Self {
        HotkeyRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct HotkeyRegistry {
    keys: HashMap<String, Hotkey>,
}

impl HotkeyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, description: &str) -> Result<(), HotkeyRegistryError> {
        if name.is_empty() {
            return Err(HotkeyRegistryError::EmptyName);
        }
        if self.keys.contains_key(name) {
            return Err(HotkeyRegistryError::Duplicate);
        }
        let h = Hotkey::new(name, description)?;
        self.keys.insert(name.to_string(), h);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Hotkey, HotkeyRegistryError> {
        self.keys.get(name).ok_or(HotkeyRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Hotkey, HotkeyRegistryError> {
        self.keys.get_mut(name).ok_or(HotkeyRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.keys.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.keys.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = HotkeyRegistry::new();
        r.add("mute", "Mute").unwrap();
        assert_eq!(r.get("mute").unwrap().name(), "mute");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = HotkeyRegistry::new();
        r.add("a", "A").unwrap();
        assert!(matches!(r.add("a", "A"), Err(HotkeyRegistryError::Duplicate)));
        assert!(matches!(r.add("", "x"), Err(HotkeyRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = HotkeyRegistry::new();
        assert!(matches!(r.get("nope"), Err(HotkeyRegistryError::Missing)));
        r.add("k", "K").unwrap();
        assert!(r.remove("k"));
        assert!(!r.contains("k"));
    }

    #[test]
    fn differential() {
        let mut r = HotkeyRegistry::new();
        r.add("start", "Start").unwrap();
        r.get_mut("start").unwrap().set_pressed(true);
        assert!(r.get("start").unwrap().pressed());
    }
}
