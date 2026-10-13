//! Safe pure LiveFeed (camera jump core).
//!
//! A feed owns a name and a source. The director jumps to it.
//! Empty names rejected. No unwraps.
//!
//! Differential: jump + current round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct LiveFeed {
    current: Option<String>,
}

impl LiveFeed {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jump(&mut self, name: &str) -> Result<(), FeedError> {
        if name.is_empty() {
            return Err(FeedError::EmptyName);
        }
        self.current = Some(name.to_string());
        Ok(())
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn clear(&mut self) {
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let f = LiveFeed::new();
        assert!(f.current().is_none());
    }

    #[test]
    fn empty_name() {
        let mut f = LiveFeed::new();
        assert!(f.jump("").is_err());
    }

    #[test]
    fn differential() {
        let mut f = LiveFeed::new();
        f.jump("trump_cam").unwrap();
        assert_eq!(f.current(), Some("trump_cam"));
        f.clear();
        assert!(f.current().is_none());
    }
}
