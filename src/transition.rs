//! Safe pure Transition (OBS scene transition core).
//!
//! A transition owns a name, a duration in milliseconds, and an
//! optional active flag. Empty names are rejected. No unwraps.
//!
//! Differential: set duration + get round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Transition {
    name: String,
    duration_ms: u32,
    active: bool,
    from: Option<String>,
    to: Option<String>,
}

impl Transition {
    pub fn new(name: &str, duration_ms: u32) -> Result<Self, TransitionError> {
        if name.is_empty() {
            return Err(TransitionError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            duration_ms,
            active: false,
            from: None,
            to: None,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    pub fn set_duration_ms(&mut self, ms: u32) {
        self.duration_ms = ms;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    /// Start a transition from one scene to another.
    pub fn start(&mut self, from: &str, to: &str) {
        self.from = Some(from.to_string());
        self.to = Some(to.to_string());
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    pub fn to(&self) -> Option<&str> {
        self.to.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = Transition::new("fade", 300).unwrap();
        assert_eq!(t.name(), "fade");
        assert_eq!(t.duration_ms(), 300);
        assert!(!t.active());
    }

    #[test]
    fn empty_name() {
        assert!(Transition::new("", 100).is_err());
    }

    #[test]
    fn differential() {
        let mut t = Transition::new("cut", 0).unwrap();
        t.set_duration_ms(500);
        t.start("Main", "Overlay");
        assert_eq!(t.duration_ms(), 500);
        assert!(t.active());
        assert_eq!(t.from(), Some("Main"));
        assert_eq!(t.to(), Some("Overlay"));
        t.stop();
        assert!(!t.active());
    }
}
