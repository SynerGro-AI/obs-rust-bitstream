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
    at_time: BTreeMap<String, String>,
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

    /// Schedule a macro at a time and date, like "2026-10-13 21:00".
    pub fn at_datetime(&mut self, name: &str, datetime: &str) -> Result<(), ScheduleError> {
        if name.is_empty() || datetime.is_empty() {
            return Err(ScheduleError::EmptyName);
        }
        self.at_time.insert(name.to_string(), datetime.to_string());
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

    /// Macros due at or before this datetime string.
    pub fn due_datetime(&self, now: &str) -> Vec<&str> {
        self.at_time
            .iter()
            .filter(|(_, t)| t.as_str() <= now)
            .map(|(n, _)| n.as_str())
            .collect()
    }

    pub fn datetime_of(&self, name: &str) -> Option<&str> {
        self.at_time.get(name).map(|s| s.as_str())
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

    #[test]
    fn datetime() {
        let mut s = Schedule::new();
        s.at_datetime("evening", "2026-10-13 21:00").unwrap();
        assert!(s.due_datetime("2026-10-13 20:00").is_empty());
        assert_eq!(s.due_datetime("2026-10-13 21:00"), vec!["evening"]);
        assert_eq!(s.datetime_of("evening"), Some("2026-10-13 21:00"));
    }
}
