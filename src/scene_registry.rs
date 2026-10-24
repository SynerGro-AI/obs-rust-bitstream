//! Safe pure SceneRegistry (OBS scene lookup core).
//!
//! Name-to-SceneContext map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + get returns the same name; items round-trip.

use std::collections::HashMap;

use crate::scene_context::{SceneContext, SceneError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneRegistryError {
    EmptyName,
    Duplicate,
    Missing,
    DuplicateItem,
}

impl From<SceneError> for SceneRegistryError {
    fn from(e: SceneError) -> Self {
        match e {
            SceneError::EmptyName => SceneRegistryError::EmptyName,
            SceneError::DuplicateItem => SceneRegistryError::DuplicateItem,
            SceneError::MissingItem => SceneRegistryError::Missing,
            SceneError::EmptySource => SceneRegistryError::EmptyName,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SceneRegistry {
    scenes: HashMap<String, SceneContext>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), SceneRegistryError> {
        if name.is_empty() {
            return Err(SceneRegistryError::EmptyName);
        }
        if self.scenes.contains_key(name) {
            return Err(SceneRegistryError::Duplicate);
        }
        let ctx = SceneContext::new(name)?;
        self.scenes.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&SceneContext, SceneRegistryError> {
        self.scenes.get(name).ok_or(SceneRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut SceneContext, SceneRegistryError> {
        self.scenes.get_mut(name).ok_or(SceneRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.scenes.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.scenes.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.scenes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = SceneRegistry::new();
        r.add("Main").unwrap();
        assert_eq!(r.get("Main").unwrap().name(), "Main");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = SceneRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(SceneRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(SceneRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = SceneRegistry::new();
        assert!(matches!(r.get("nope"), Err(SceneRegistryError::Missing)));
        r.add("s").unwrap();
        assert!(r.remove("s"));
        assert!(!r.contains("s"));
    }

    #[test]
    fn differential() {
        let mut r = SceneRegistry::new();
        r.add("scene").unwrap();
        r.get_mut("scene")
            .unwrap()
            .add_item("cam", "camera_src")
            .unwrap();
        assert!(r.get("scene").unwrap().contains("cam"));
    }
}
