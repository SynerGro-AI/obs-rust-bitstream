//! Safe in-memory config store (OBS config-file.h core).
//!
//! The C version is an ini-style file with sections, values, and defaults.
//! The safe path is a pure HashMap of sections → keys → values, with a
//! separate defaults map. No file I/O, no null derefs, no unwraps.
//!
//! Differential: set then get returns the same string; missing keys
//! fall back to defaults exactly as documented.

use std::collections::HashMap;
#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
    defaults: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    fn section_mut<'a>(map: &'a mut HashMap<String, HashMap<String, String>>, section: &str) -> &'a mut HashMap<String, String> {
        map.entry(section.to_string()).or_default()
    }

    pub fn set_string(&mut self, section: &str, name: &str, value: &str) {
        Self::section_mut(&mut self.sections, section).insert(name.to_string(), value.to_string());
    }

    pub fn set_default_string(&mut self, section: &str, name: &str, value: &str) {
        Self::section_mut(&mut self.defaults, section).insert(name.to_string(), value.to_string());
    }

    pub fn get_string(&self, section: &str, name: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|s| s.get(name))
            .or_else(|| self.defaults.get(section).and_then(|d| d.get(name)))
            .map(|s| s.as_str())
    }

    pub fn has_user_value(&self, section: &str, name: &str) -> bool {
        self.sections.get(section).and_then(|s| s.get(name)).is_some()
    }

    pub fn remove_value(&mut self, section: &str, name: &str) -> bool {
        self.sections
            .get_mut(section)
            .and_then(|s| s.remove(name))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut c = Config::new();
        c.set_string("Video", "BaseCX", "1920");
        assert_eq!(c.get_string("Video", "BaseCX"), Some("1920"));
    }

    #[test]
    fn defaults_fallback() {
        let mut c = Config::new();
        c.set_default_string("Video", "FPS", "30");
        assert_eq!(c.get_string("Video", "FPS"), Some("30"));
        c.set_string("Video", "FPS", "60");
        assert_eq!(c.get_string("Video", "FPS"), Some("60"));
    }

    #[test]
    fn missing_returns_none() {
        let c = Config::new();
        assert_eq!(c.get_string("Missing", "Key"), None);
    }

    #[test]
    fn differential() {
        let mut safe = Config::new();
        safe.set_string("s", "k", "v");
        let got = safe.get_string("s", "k").unwrap_or("");
        assert_equal(got, "v", "config get");
    }
}
