//! Safe pure Macro (key sequence core).
//!
//! Records a sequence of keys and replays them. Empty
//! names rejected. No unwraps.
//!
//! Differential: record + replay round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct Macro {
    macros: BTreeMap<String, Vec<String>>,
    recording: Option<String>,
}

impl Macro {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start recording a macro under a name.
    pub fn start(&mut self, name: &str) -> Result<(), MacroError> {
        if name.is_empty() {
            return Err(MacroError::EmptyName);
        }
        self.recording = Some(name.to_string());
        self.macros.insert(name.to_string(), Vec::new());
        Ok(())
    }

    /// Record a key into the current macro.
    pub fn record(&mut self, key: &str) {
        if let Some(name) = &self.recording {
            if let Some(seq) = self.macros.get_mut(name) {
                seq.push(key.to_string());
            }
        }
    }

    pub fn stop(&mut self) {
        self.recording = None;
    }

    /// The recorded sequence for a macro.
    pub fn sequence(&self, name: &str) -> Option<&[String]> {
        self.macros.get(name).map(|v| v.as_slice())
    }

    pub fn count(&self) -> usize {
        self.macros.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = Macro::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut m = Macro::new();
        assert!(m.start("").is_err());
    }

    #[test]
    fn differential() {
        let mut m = Macro::new();
        m.start("go_live").unwrap();
        m.record("r");
        m.record("s");
        m.record("v");
        m.stop();
        assert_eq!(m.sequence("go_live"), Some(&["r".to_string(), "s".to_string(), "v".to_string()][..]));
    }
}
