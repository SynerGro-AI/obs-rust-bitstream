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

    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Cancel a recorded macro. Returns true if it existed.
    pub fn cancel(&mut self, name: &str) -> bool {
        self.macros.remove(name).is_some()
    }

    /// The recorded sequence for a macro.
    pub fn sequence(&self, name: &str) -> Option<&[String]> {
        self.macros.get(name).map(|v| v.as_slice())
    }

    /// Replace the sequence of a macro.
    pub fn edit(&mut self, name: &str, keys: &[&str]) -> Result<(), MacroError> {
        if !self.macros.contains_key(name) {
            return Err(MacroError::Missing);
        }
        self.macros.insert(
            name.to_string(),
            keys.iter().map(|s| s.to_string()).collect(),
        );
        Ok(())
    }

    /// Rename a macro.
    pub fn rename(&mut self, from: &str, to: &str) -> Result<(), MacroError> {
        if to.is_empty() {
            return Err(MacroError::EmptyName);
        }
        let seq = self.macros.remove(from).ok_or(MacroError::Missing)?;
        self.macros.insert(to.to_string(), seq);
        Ok(())
    }

    /// All macro names.
    pub fn names(&self) -> Vec<&str> {
        self.macros.keys().map(|s| s.as_str()).collect()
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
        assert_eq!(
            m.sequence("go_live"),
            Some(&["r".to_string(), "s".to_string(), "v".to_string()][..])
        );
    }

    #[test]
    fn cancel() {
        let mut m = Macro::new();
        m.start("go_live").unwrap();
        m.stop();
        assert!(m.cancel("go_live"));
        assert!(m.sequence("go_live").is_none());
    }

    #[test]
    fn edit_and_rename() {
        let mut m = Macro::new();
        m.start("go_live").unwrap();
        m.record("r");
        m.stop();
        m.edit("go_live", &["s", "v"]).unwrap();
        assert_eq!(m.sequence("go_live").unwrap().len(), 2);
        m.rename("go_live", "broadcast").unwrap();
        assert!(m.sequence("go_live").is_none());
        assert_eq!(m.names(), vec!["broadcast"]);
    }
}
