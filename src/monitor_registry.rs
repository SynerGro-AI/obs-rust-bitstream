//! Safe pure MonitorRegistry (OBS audio monitor lookup core).
//!
//! Name-to-AudioMonitor map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + set volume through the registry round-trips.

use std::collections::HashMap;

use crate::audio_monitor::{AudioMonitor, MonitorError};

#[derive(Clone, Debug, PartialEq)]
pub enum MonitorRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<MonitorError> for MonitorRegistryError {
    fn from(_: MonitorError) -> Self {
        MonitorRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct MonitorRegistry {
    monitors: HashMap<String, AudioMonitor>,
}

impl MonitorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), MonitorRegistryError> {
        if name.is_empty() {
            return Err(MonitorRegistryError::EmptyName);
        }
        if self.monitors.contains_key(name) {
            return Err(MonitorRegistryError::Duplicate);
        }
        let m = AudioMonitor::new(name)?;
        self.monitors.insert(name.to_string(), m);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&AudioMonitor, MonitorRegistryError> {
        self.monitors.get(name).ok_or(MonitorRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut AudioMonitor, MonitorRegistryError> {
        self.monitors
            .get_mut(name)
            .ok_or(MonitorRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.monitors.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.monitors.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.monitors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = MonitorRegistry::new();
        r.add("desktop").unwrap();
        assert_eq!(r.get("desktop").unwrap().name(), "desktop");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = MonitorRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(MonitorRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(MonitorRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = MonitorRegistry::new();
        assert!(matches!(r.get("nope"), Err(MonitorRegistryError::Missing)));
        r.add("m").unwrap();
        assert!(r.remove("m"));
        assert!(!r.contains("m"));
    }

    #[test]
    fn differential() {
        let mut r = MonitorRegistry::new();
        r.add("mic").unwrap();
        r.get_mut("mic").unwrap().set_volume(0.25);
        r.get_mut("mic").unwrap().set_muted(true);
        assert_eq!(r.get("mic").unwrap().gain(), 0.0);
    }
}
