//! Safe pure StreamingRegistry (OBS streaming lookup core).
//!
//! Name-to-Streaming map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + start through the registry round-trips.

use std::collections::HashMap;

use crate::streaming::{Streaming, StreamingError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamingRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<StreamingError> for StreamingRegistryError {
    fn from(_: StreamingError) -> Self {
        StreamingRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct StreamingRegistry {
    streams: HashMap<String, Streaming>,
}

impl StreamingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), StreamingRegistryError> {
        if name.is_empty() {
            return Err(StreamingRegistryError::EmptyName);
        }
        if self.streams.contains_key(name) {
            return Err(StreamingRegistryError::Duplicate);
        }
        let s = Streaming::new(name)?;
        self.streams.insert(name.to_string(), s);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Streaming, StreamingRegistryError> {
        self.streams
            .get(name)
            .ok_or(StreamingRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Streaming, StreamingRegistryError> {
        self.streams
            .get_mut(name)
            .ok_or(StreamingRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.streams.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.streams.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.streams.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = StreamingRegistry::new();
        r.add("main").unwrap();
        assert_eq!(r.get("main").unwrap().name(), "main");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = StreamingRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(StreamingRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(StreamingRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = StreamingRegistry::new();
        assert!(matches!(
            r.get("nope"),
            Err(StreamingRegistryError::Missing)
        ));
        r.add("s").unwrap();
        assert!(r.remove("s"));
        assert!(!r.contains("s"));
    }

    #[test]
    fn differential() {
        let mut r = StreamingRegistry::new();
        r.add("stream").unwrap();
        r.get_mut("stream").unwrap().set_service("Twitch");
        r.get_mut("stream").unwrap().start();
        assert!(r.get("stream").unwrap().active());
        assert_eq!(r.get("stream").unwrap().service(), "Twitch");
    }
}
