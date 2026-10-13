//! Safe pure Keybind (key to action core).
//!
//! Maps a key to an action name. Empty keys rejected.
//! No unwraps.
//!
//! Differential: bind + lookup round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeybindError {
    EmptyKey,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct Keybind {
    binds: BTreeMap<String, String>,
}

impl Keybind {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, key: &str, action: &str) -> Result<(), KeybindError> {
        if key.is_empty() || action.is_empty() {
            return Err(KeybindError::EmptyKey);
        }
        self.binds.insert(key.to_string(), action.to_string());
        Ok(())
    }

    pub fn action(&self, key: &str) -> Option<&str> {
        self.binds.get(key).map(|s| s.as_str())
    }

    pub fn count(&self) -> usize {
        self.binds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let h = Keybind::new();
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn empty_key() {
        let mut h = Keybind::new();
        assert!(h.bind("", "record").is_err());
    }

    #[test]
    fn differential() {
        let mut h = Keybind::new();
        h.bind("r", "record").unwrap();
        h.bind("s", "stream").unwrap();
        h.bind("v", "vcam").unwrap();
        assert_eq!(h.action("s"), Some("stream"));
        assert_eq!(h.count(), 3);
    }
}
