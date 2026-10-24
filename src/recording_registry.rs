//! Safe pure RecordingRegistry (OBS recording lookup core).
//!
//! Name-to-Recording map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + start through the registry round-trips.

use std::collections::HashMap;

use crate::recording::{Recording, RecordingError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordingRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<RecordingError> for RecordingRegistryError {
    fn from(_: RecordingError) -> Self {
        RecordingRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct RecordingRegistry {
    recordings: HashMap<String, Recording>,
}

impl RecordingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), RecordingRegistryError> {
        if name.is_empty() {
            return Err(RecordingRegistryError::EmptyName);
        }
        if self.recordings.contains_key(name) {
            return Err(RecordingRegistryError::Duplicate);
        }
        let r = Recording::new(name)?;
        self.recordings.insert(name.to_string(), r);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Recording, RecordingRegistryError> {
        self.recordings
            .get(name)
            .ok_or(RecordingRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Recording, RecordingRegistryError> {
        self.recordings
            .get_mut(name)
            .ok_or(RecordingRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.recordings.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.recordings.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.recordings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = RecordingRegistry::new();
        r.add("main").unwrap();
        assert_eq!(r.get("main").unwrap().name(), "main");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = RecordingRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(RecordingRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(RecordingRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = RecordingRegistry::new();
        assert!(matches!(
            r.get("nope"),
            Err(RecordingRegistryError::Missing)
        ));
        r.add("r").unwrap();
        assert!(r.remove("r"));
        assert!(!r.contains("r"));
    }

    #[test]
    fn differential() {
        let mut r = RecordingRegistry::new();
        r.add("rec").unwrap();
        r.get_mut("rec").unwrap().set_path("/tmp/out.mp4");
        r.get_mut("rec").unwrap().start();
        assert!(r.get("rec").unwrap().active());
        assert_eq!(r.get("rec").unwrap().path(), "/tmp/out.mp4");
    }
}
