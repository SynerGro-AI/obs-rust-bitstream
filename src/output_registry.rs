//! Safe pure OutputRegistry (OBS output lookup core).
//!
//! Parallel to SourceRegistry. Name-to-OutputContext map.
//! Empty and duplicate names are rejected. No unwraps.
//!
//! Differential: add + get returns the same name; settings round-trip.

use std::collections::HashMap;

use crate::output_context::{OutputContext, OutputError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<OutputError> for OutputRegistryError {
    fn from(_: OutputError) -> Self {
        OutputRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct OutputRegistry {
    outputs: HashMap<String, OutputContext>,
}

impl OutputRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), OutputRegistryError> {
        if name.is_empty() {
            return Err(OutputRegistryError::EmptyName);
        }
        if self.outputs.contains_key(name) {
            return Err(OutputRegistryError::Duplicate);
        }
        let ctx = OutputContext::new(name)?;
        self.outputs.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&OutputContext, OutputRegistryError> {
        self.outputs.get(name).ok_or(OutputRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut OutputContext, OutputRegistryError> {
        self.outputs
            .get_mut(name)
            .ok_or(OutputRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.outputs.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.outputs.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = OutputRegistry::new();
        r.add("stream").unwrap();
        assert_eq!(r.get("stream").unwrap().name(), "stream");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = OutputRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(OutputRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(OutputRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = OutputRegistry::new();
        assert!(matches!(r.get("nope"), Err(OutputRegistryError::Missing)));
        r.add("o").unwrap();
        assert!(r.remove("o"));
        assert!(!r.contains("o"));
    }

    #[test]
    fn differential() {
        let mut r = OutputRegistry::new();
        r.add("out").unwrap();
        r.get_mut("out")
            .unwrap()
            .settings_mut()
            .set_string("url", "rtmp://x")
            .unwrap();
        assert_eq!(
            r.get("out").unwrap().settings().get_string("url").unwrap(),
            "rtmp://x"
        );
    }
}
