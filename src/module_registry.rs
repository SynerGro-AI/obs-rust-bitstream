//! Safe pure ModuleRegistry (OBS loaded modules core).
//!
//! Name-to-Module map. Empty and duplicate ids are rejected.
//! No unwraps.
//!
//! Differential: add + get returns the same module; a registered
//! source id is visible through the registry.

use std::collections::HashMap;

use crate::module::{Module, ModuleError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleRegistryError {
    EmptyId,
    Duplicate,
    Missing,
}

impl From<ModuleError> for ModuleRegistryError {
    fn from(e: ModuleError) -> Self {
        match e {
            ModuleError::EmptyId => ModuleRegistryError::EmptyId,
            ModuleError::Duplicate => ModuleRegistryError::Duplicate,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: &str) -> Result<(), ModuleRegistryError> {
        if id.is_empty() {
            return Err(ModuleRegistryError::EmptyId);
        }
        if self.modules.contains_key(id) {
            return Err(ModuleRegistryError::Duplicate);
        }
        let m = Module::new(id)?;
        self.modules.insert(id.to_string(), m);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&Module, ModuleRegistryError> {
        self.modules.get(id).ok_or(ModuleRegistryError::Missing)
    }

    pub fn get_mut(&mut self, id: &str) -> Result<&mut Module, ModuleRegistryError> {
        self.modules.get_mut(id).ok_or(ModuleRegistryError::Missing)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.modules.remove(id).is_some()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.modules.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = ModuleRegistry::new();
        r.add("obs-ffmpeg").unwrap();
        assert_eq!(r.get("obs-ffmpeg").unwrap().id(), "obs-ffmpeg");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = ModuleRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(ModuleRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(ModuleRegistryError::EmptyId)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = ModuleRegistry::new();
        assert!(matches!(r.get("nope"), Err(ModuleRegistryError::Missing)));
        r.add("m").unwrap();
        assert!(r.remove("m"));
        assert!(!r.contains("m"));
    }

    #[test]
    fn differential() {
        let mut r = ModuleRegistry::new();
        r.add("obs-ffmpeg").unwrap();
        r.get_mut("obs-ffmpeg").unwrap().register_source("ffmpeg_source").unwrap();
        assert!(r.get("obs-ffmpeg").unwrap().has_source("ffmpeg_source"));
    }
}
