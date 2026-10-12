//! Safe pure ReplayRegistry (OBS replay buffer lookup core).
//!
//! Name-to-ReplayBuffer map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + save through the registry round-trips.

use std::collections::HashMap;

use crate::replay_buffer::{ReplayBuffer, ReplayError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<ReplayError> for ReplayRegistryError {
    fn from(_: ReplayError) -> Self {
        ReplayRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReplayRegistry {
    buffers: HashMap<String, ReplayBuffer>,
}

impl ReplayRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, duration_secs: u32) -> Result<(), ReplayRegistryError> {
        if name.is_empty() {
            return Err(ReplayRegistryError::EmptyName);
        }
        if self.buffers.contains_key(name) {
            return Err(ReplayRegistryError::Duplicate);
        }
        let b = ReplayBuffer::new(name, duration_secs)?;
        self.buffers.insert(name.to_string(), b);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&ReplayBuffer, ReplayRegistryError> {
        self.buffers.get(name).ok_or(ReplayRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut ReplayBuffer, ReplayRegistryError> {
        self.buffers.get_mut(name).ok_or(ReplayRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.buffers.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.buffers.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.buffers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = ReplayRegistry::new();
        r.add("main", 20).unwrap();
        assert_eq!(r.get("main").unwrap().duration_secs(), 20);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = ReplayRegistry::new();
        r.add("a", 10).unwrap();
        assert!(matches!(r.add("a", 10), Err(ReplayRegistryError::Duplicate)));
        assert!(matches!(r.add("", 10), Err(ReplayRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = ReplayRegistry::new();
        assert!(matches!(r.get("nope"), Err(ReplayRegistryError::Missing)));
        r.add("b", 10).unwrap();
        assert!(r.remove("b"));
        assert!(!r.contains("b"));
    }

    #[test]
    fn differential() {
        let mut r = ReplayRegistry::new();
        r.add("buf", 30).unwrap();
        r.get_mut("buf").unwrap().start();
        assert!(r.get_mut("buf").unwrap().save());
        assert!(r.get("buf").unwrap().saved());
    }
}
