//! Safe pure Recording (OBS recording core).
//!
//! A recording owns a name, an output path, and an active flag.
//! Empty names are rejected. No unwraps.
//!
//! Differential: start + is active round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordingError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Recording {
    name: String,
    path: String,
    active: bool,
}

impl Recording {
    pub fn new(name: &str) -> Result<Self, RecordingError> {
        if name.is_empty() {
            return Err(RecordingError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            path: String::new(),
            active: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = Recording::new("main").unwrap();
        assert_eq!(r.name(), "main");
        assert!(!r.active());
    }

    #[test]
    fn empty_name() {
        assert!(Recording::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut r = Recording::new("rec").unwrap();
        r.set_path("/tmp/out.mp4");
        r.start();
        assert_eq!(r.path(), "/tmp/out.mp4");
        assert!(r.active());
        r.stop();
        assert!(!r.active());
    }
}
