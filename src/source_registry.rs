//! Safe pure SourceRegistry (OBS source lookup core).
//!
//! The C version is a name-to-source map used by the core.
//! The safe path is a HashMap of name to SourceContext.
//! Duplicate names and empty names are rejected. No unwraps.
//!
//! Differential: add + get returns the same name.

use std::collections::HashMap;

use crate::source_context::{SourceContext, SourceError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<SourceError> for RegistryError {
    fn from(_: SourceError) -> Self {
        RegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct SourceRegistry {
    sources: HashMap<String, SourceContext>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a source by name. Rejects empty and duplicate names.
    pub fn add(&mut self, name: &str) -> Result<(), RegistryError> {
        if name.is_empty() {
            return Err(RegistryError::EmptyName);
        }
        if self.sources.contains_key(name) {
            return Err(RegistryError::Duplicate);
        }
        let ctx = SourceContext::new(name)?;
        self.sources.insert(name.to_string(), ctx);
        Ok(())
    }

    /// Add a source with a type id. Rejects empty and duplicate names.
    pub fn add_with_type(&mut self, name: &str, type_id: &str) -> Result<(), RegistryError> {
        if name.is_empty() {
            return Err(RegistryError::EmptyName);
        }
        if self.sources.contains_key(name) {
            return Err(RegistryError::Duplicate);
        }
        let ctx = SourceContext::with_type(name, type_id)?;
        self.sources.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&SourceContext, RegistryError> {
        self.sources.get(name).ok_or(RegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut SourceContext, RegistryError> {
        self.sources.get_mut(name).ok_or(RegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.sources.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.sources.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn names(&self) -> Vec<&str> {
        self.sources.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = SourceRegistry::new();
        r.add("Camera").unwrap();
        assert_eq!(r.get("Camera").unwrap().name(), "Camera");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = SourceRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(RegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(RegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = SourceRegistry::new();
        assert!(matches!(r.get("nope"), Err(RegistryError::Missing)));
        r.add("s").unwrap();
        assert!(r.remove("s"));
        assert!(!r.contains("s"));
    }

    #[test]
    fn differential() {
        let mut r = SourceRegistry::new();
        r.add("src").unwrap();
        r.get_mut("src").unwrap().settings_mut().set_int("w", 1280).unwrap();
        assert_eq!(r.get("src").unwrap().settings().get_int("w").unwrap(), 1280);
    }
}
