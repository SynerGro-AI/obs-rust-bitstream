//! Safe pure MultiMonitor (screen layout core).
//!
//! Maps a monitor name to the scene it shows. Empty names
//! rejected. No unwraps.
//!
//! Differential: assign + read round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct MultiMonitor {
    screens: BTreeMap<String, String>,
}

impl MultiMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assign(&mut self, monitor: &str, scene: &str) -> Result<(), ScreenError> {
        if monitor.is_empty() || scene.is_empty() {
            return Err(ScreenError::EmptyName);
        }
        self.screens.insert(monitor.to_string(), scene.to_string());
        Ok(())
    }

    pub fn showing(&self, monitor: &str) -> Option<&str> {
        self.screens.get(monitor).map(|s| s.as_str())
    }

    pub fn count(&self) -> usize {
        self.screens.len()
    }

    pub fn names(&self) -> Vec<&str> {
        self.screens.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = MultiMonitor::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut m = MultiMonitor::new();
        assert!(m.assign("", "wide").is_err());
    }

    #[test]
    fn differential() {
        let mut m = MultiMonitor::new();
        m.assign("program", "wide").unwrap();
        m.assign("preview", "close").unwrap();
        m.assign("multiview", "grid").unwrap();
        assert_eq!(m.count(), 3);
        assert_eq!(m.showing("preview"), Some("close"));
    }
}
