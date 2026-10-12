//! Safe pure Streaming (OBS streaming core).
//!
//! A stream owns a name, a service name, and an active flag.
//! Empty names are rejected. No unwraps.
//!
//! Differential: start + is active round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamingError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Streaming {
    name: String,
    service: String,
    active: bool,
}

impl Streaming {
    pub fn new(name: &str) -> Result<Self, StreamingError> {
        if name.is_empty() {
            return Err(StreamingError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            service: String::new(),
            active: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn set_service(&mut self, service: &str) {
        self.service = service.to_string();
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
        let s = Streaming::new("main").unwrap();
        assert_eq!(s.name(), "main");
        assert!(!s.active());
    }

    #[test]
    fn empty_name() {
        assert!(Streaming::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = Streaming::new("stream").unwrap();
        s.set_service("Twitch");
        s.start();
        assert_eq!(s.service(), "Twitch");
        assert!(s.active());
        s.stop();
        assert!(!s.active());
    }
}
