//! Safe pure Presence (online members core).
//!
//! Tracks who is online. Empty names rejected.
//! No unwraps.
//!
//! Differential: join + leave round-trips.

use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresenceError {
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct Presence {
    online: BTreeSet<String>,
}

impl Presence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn join(&mut self, name: &str) -> Result<(), PresenceError> {
        if name.is_empty() {
            return Err(PresenceError::EmptyName);
        }
        self.online.insert(name.to_string());
        Ok(())
    }

    pub fn leave(&mut self, name: &str) {
        self.online.remove(name);
    }

    pub fn is_online(&self, name: &str) -> bool {
        self.online.contains(name)
    }

    pub fn online(&self) -> Vec<&str> {
        self.online.iter().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.online.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut p = Presence::new();
        assert!(p.join("").is_err());
    }

    #[test]
    fn differential() {
        let mut p = Presence::new();
        p.join("Jason").unwrap();
        p.join("Alex").unwrap();
        p.join("Jason").unwrap();
        assert_eq!(p.count(), 2);
        assert!(p.is_online("Alex"));
        p.leave("Alex");
        assert_eq!(p.online(), vec!["Jason"]);
    }
}
