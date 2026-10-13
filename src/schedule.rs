//! Safe pure Schedule (timed macro core).
//!
//! Runs a macro at a frame number. Empty names rejected.
//! No unwraps.
//!
//! Differential: a macro due at frame 10 fires at frame 10.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct Schedule {
    due: BTreeMap<String, u64>,
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedule a macro to run at a frame.
    pub fn at(&mut self, name: &str, frame: u64) -> Result<(), ScheduleError> {
        if name.is_empty() {
            return Err(ScheduleError::EmptyName);
        }
        self.due.insert(name.to_string(), frame);
        Ok(())
    }

    /// Macros due at or before this frame.
    pub fn due_at(&self, frame: u64) -> Vec<&str> {
        self.due
            .iter()
            .filter(|(_, f)| **f <= frame)
            .map(|(n, _)| n.as_str())
            .collect()
    }

    pub fn count(&self) -> usize {
        self.due.len()
    }

    pub fn clear(&mut self, name: &str) {
        self.due.remove(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = Schedule::new();
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut s = Schedule::new();
        assert!(s.at("", 10).is_err());
    }

    #[test]
    fn differential() {
        let mut s = Schedule::new();
        s.at("go_live", 10).unwrap();
        assert!(s.due_at(9).is_empty());
        assert_eq!(s.due_at(10), vec!["go_live"]);
    }
}
