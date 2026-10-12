//! Safe pure ReplayBuffer (OBS replay buffer core).
//!
//! A buffer owns a name, a duration in seconds, and an active flag.
//! Empty names are rejected. No unwraps.
//!
//! Differential: set duration + start round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct ReplayBuffer {
    name: String,
    duration_secs: u32,
    active: bool,
    saved: bool,
}

impl ReplayBuffer {
    pub fn new(name: &str, duration_secs: u32) -> Result<Self, ReplayError> {
        if name.is_empty() {
            return Err(ReplayError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            duration_secs,
            active: false,
            saved: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration_secs(&self) -> u32 {
        self.duration_secs
    }

    pub fn set_duration_secs(&mut self, secs: u32) {
        self.duration_secs = secs;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        self.active = true;
        self.saved = false;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Save the buffer. Returns true if it was active.
    pub fn save(&mut self) -> bool {
        if self.active {
            self.saved = true;
            true
        } else {
            false
        }
    }

    pub fn saved(&self) -> bool {
        self.saved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = ReplayBuffer::new("main", 20).unwrap();
        assert_eq!(r.duration_secs(), 20);
        assert!(!r.active());
    }

    #[test]
    fn empty_name() {
        assert!(ReplayBuffer::new("", 10).is_err());
    }

    #[test]
    fn differential() {
        let mut r = ReplayBuffer::new("buf", 30).unwrap();
        r.set_duration_secs(15);
        r.start();
        assert!(r.save());
        assert!(r.saved());
        assert_eq!(r.duration_secs(), 15);
        r.stop();
        assert!(!r.active());
    }
}
