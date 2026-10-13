//! Safe pure OptionsMenu (monitor camera options core).
//!
//! Lists monitors and assigns a camera to each. Empty names
//! rejected. No unwraps.
//!
//! Differential: set camera + read round-trips.

use crate::multi_monitor::MultiMonitor;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct OptionsMenu {
    cameras: BTreeMap<String, String>,
}

impl OptionsMenu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign a camera to a monitor.
    pub fn set_camera(&mut self, monitor: &str, camera: &str) -> Result<(), MenuError> {
        if monitor.is_empty() || camera.is_empty() {
            return Err(MenuError::EmptyName);
        }
        self.cameras.insert(monitor.to_string(), camera.to_string());
        Ok(())
    }

    pub fn camera_of(&self, monitor: &str) -> Option<&str> {
        self.cameras.get(monitor).map(|s| s.as_str())
    }

    /// Monitors that have a camera assigned.
    pub fn monitors(&self) -> Vec<&str> {
        self.cameras.keys().map(|s| s.as_str()).collect()
    }

    /// Apply the menu to a multi-monitor layout. Each monitor
    /// shows its assigned camera as the scene.
    pub fn apply(&self, screens: &mut MultiMonitor) {
        for (monitor, camera) in &self.cameras {
            let _ = screens.assign(monitor, camera);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = OptionsMenu::new();
        assert!(m.monitors().is_empty());
    }

    #[test]
    fn empty_name() {
        let mut m = OptionsMenu::new();
        assert!(m.set_camera("", "cam1").is_err());
    }

    #[test]
    fn differential() {
        let mut menu = OptionsMenu::new();
        menu.set_camera("program", "cam1").unwrap();
        menu.set_camera("preview", "cam2").unwrap();
        assert_eq!(menu.camera_of("program"), Some("cam1"));
        let mut screens = MultiMonitor::new();
        menu.apply(&mut screens);
        assert_eq!(screens.showing("preview"), Some("cam2"));
    }
}
