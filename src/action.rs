//! Safe pure Action (detected action core).
//!
//! Holds the last detected action. The director reads it.
//! No unwraps.
//!
//! Differential: detect an action + last round-trips.

#[derive(Clone, Debug, Default)]
pub struct Action {
    last: String,
}

impl Action {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn detect(&mut self, action: &str) {
        self.last = action.to_string();
    }

    pub fn last(&self) -> &str {
        &self.last
    }

    pub fn is_empty(&self) -> bool {
        self.last.is_empty()
    }

    pub fn clear(&mut self) {
        self.last.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let a = Action::new();
        assert!(a.is_empty());
    }

    #[test]
    fn differential() {
        let mut a = Action::new();
        a.detect("shootout");
        assert_eq!(a.last(), "shootout");
        a.clear();
        assert!(a.is_empty());
    }
}
