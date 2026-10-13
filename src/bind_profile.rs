//! Safe pure BindProfile (keybind persistence core).
//!
//! Saves and loads key bindings. Empty names rejected.
//! No unwraps.
//!
//! Differential: save + load round-trips.

use crate::keybind::Keybind;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindProfileError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct BindProfile {
    profiles: BTreeMap<String, BTreeMap<String, String>>,
}

impl BindProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save the current binds under a name.
    pub fn save(&mut self, name: &str, binds: &Keybind) -> Result<(), BindProfileError> {
        if name.is_empty() {
            return Err(BindProfileError::EmptyName);
        }
        let mut map = BTreeMap::new();
        for key in binds.keys() {
            if let Some(action) = binds.action(key) {
                map.insert(key.to_string(), action.to_string());
            }
        }
        self.profiles.insert(name.to_string(), map);
        Ok(())
    }

    /// Load binds from a named profile.
    pub fn load(&self, name: &str, binds: &mut Keybind) -> Result<(), BindProfileError> {
        let map = self.profiles.get(name).ok_or(BindProfileError::Missing)?;
        for (key, action) in map {
            let _ = binds.bind(key, action);
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.profiles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let p = BindProfile::new();
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut p = BindProfile::new();
        let b = Keybind::new();
        assert!(p.save("", &b).is_err());
    }

    #[test]
    fn differential() {
        let mut p = BindProfile::new();
        let mut b = Keybind::new();
        b.bind("q", "record").unwrap();
        p.save("mine", &b).unwrap();
        let mut fresh = Keybind::new();
        p.load("mine", &mut fresh).unwrap();
        assert_eq!(fresh.action("q"), Some("record"));
    }
}
