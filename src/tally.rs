//! Safe pure Tally (live indicator core).
//!
//! Marks which camera is live. Empty names rejected.
//! No unwraps.
//!
//! Differential: set live + is_live round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TallyError {
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct Tally {
    live: Option<String>,
}

impl Tally {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_live(&mut self, camera: &str) -> Result<(), TallyError> {
        if camera.is_empty() {
            return Err(TallyError::EmptyName);
        }
        self.live = Some(camera.to_string());
        Ok(())
    }

    pub fn is_live(&self, camera: &str) -> bool {
        self.live.as_deref() == Some(camera)
    }

    pub fn live(&self) -> Option<&str> {
        self.live.as_deref()
    }

    pub fn clear(&mut self) {
        self.live = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = Tally::new();
        assert!(t.live().is_none());
    }

    #[test]
    fn empty_name() {
        let mut t = Tally::new();
        assert!(t.set_live("").is_err());
    }

    #[test]
    fn differential() {
        let mut t = Tally::new();
        t.set_live("cam2").unwrap();
        assert!(t.is_live("cam2"));
        assert!(!t.is_live("cam1"));
        t.clear();
        assert!(t.live().is_none());
    }
}
