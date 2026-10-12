//! Safe pure CollectionRegistry (OBS scene collection lookup core).
//!
//! Name-to-SceneCollection map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + get returns the same collection; a scene
//! added through the registry is visible.

use std::collections::HashMap;

use crate::scene_collection::{CollectionError, SceneCollection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<CollectionError> for CollectionRegistryError {
    fn from(e: CollectionError) -> Self {
        match e {
            CollectionError::EmptyName => CollectionRegistryError::EmptyName,
            CollectionError::Duplicate => CollectionRegistryError::Duplicate,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CollectionRegistry {
    collections: HashMap<String, SceneCollection>,
}

impl CollectionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), CollectionRegistryError> {
        if name.is_empty() {
            return Err(CollectionRegistryError::EmptyName);
        }
        if self.collections.contains_key(name) {
            return Err(CollectionRegistryError::Duplicate);
        }
        let c = SceneCollection::new(name)?;
        self.collections.insert(name.to_string(), c);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&SceneCollection, CollectionRegistryError> {
        self.collections.get(name).ok_or(CollectionRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut SceneCollection, CollectionRegistryError> {
        self.collections.get_mut(name).ok_or(CollectionRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.collections.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.collections.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.collections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = CollectionRegistry::new();
        r.add("Default").unwrap();
        assert_eq!(r.get("Default").unwrap().name(), "Default");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = CollectionRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(CollectionRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(CollectionRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = CollectionRegistry::new();
        assert!(matches!(r.get("nope"), Err(CollectionRegistryError::Missing)));
        r.add("c").unwrap();
        assert!(r.remove("c"));
        assert!(!r.contains("c"));
    }

    #[test]
    fn differential() {
        let mut r = CollectionRegistry::new();
        r.add("col").unwrap();
        r.get_mut("col").unwrap().add_scene("Main").unwrap();
        assert!(r.get("col").unwrap().contains("Main"));
    }
}
