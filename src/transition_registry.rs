//! Safe pure TransitionRegistry (OBS transition lookup core).
//!
//! Name-to-Transition map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + start through the registry round-trips.

use std::collections::HashMap;

use crate::transition::{Transition, TransitionError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<TransitionError> for TransitionRegistryError {
    fn from(_: TransitionError) -> Self {
        TransitionRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct TransitionRegistry {
    transitions: HashMap<String, Transition>,
}

impl TransitionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, duration_ms: u32) -> Result<(), TransitionRegistryError> {
        if name.is_empty() {
            return Err(TransitionRegistryError::EmptyName);
        }
        if self.transitions.contains_key(name) {
            return Err(TransitionRegistryError::Duplicate);
        }
        let t = Transition::new(name, duration_ms)?;
        self.transitions.insert(name.to_string(), t);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Transition, TransitionRegistryError> {
        self.transitions
            .get(name)
            .ok_or(TransitionRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Transition, TransitionRegistryError> {
        self.transitions
            .get_mut(name)
            .ok_or(TransitionRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.transitions.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.transitions.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = TransitionRegistry::new();
        r.add("fade", 300).unwrap();
        assert_eq!(r.get("fade").unwrap().duration_ms(), 300);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = TransitionRegistry::new();
        r.add("a", 0).unwrap();
        assert!(matches!(
            r.add("a", 0),
            Err(TransitionRegistryError::Duplicate)
        ));
        assert!(matches!(
            r.add("", 0),
            Err(TransitionRegistryError::EmptyName)
        ));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = TransitionRegistry::new();
        assert!(matches!(
            r.get("nope"),
            Err(TransitionRegistryError::Missing)
        ));
        r.add("t", 0).unwrap();
        assert!(r.remove("t"));
        assert!(!r.contains("t"));
    }

    #[test]
    fn differential() {
        let mut r = TransitionRegistry::new();
        r.add("cut", 0).unwrap();
        r.get_mut("cut").unwrap().start("Main", "Overlay");
        assert!(r.get("cut").unwrap().active());
        assert_eq!(r.get("cut").unwrap().to(), Some("Overlay"));
    }
}
