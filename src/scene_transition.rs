//! Safe pure SceneTransition (from one scene to another).
//!
//! Records the from and to names. No unwraps.
//!
//! Differential: a known pair produces the expected summary.

#[derive(Clone, Debug)]
pub struct SceneTransition {
    from: String,
    to: String,
}

impl SceneTransition {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    /// True if this is a no-op.
    pub fn is_same(&self) -> bool {
        self.from == self.to
    }

    /// One line for the transition.
    pub fn summary(&self) -> String {
        if self.is_same() {
            format!("stay {}", self.from)
        } else {
            format!("{} -> {}", self.from, self.to)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let t = SceneTransition::new("main", "break");
        assert_eq!(t.from(), "main");
        assert_eq!(t.to(), "break");
        assert!(!t.is_same());
        assert!(t.summary().contains("main -> break"));
        let same = SceneTransition::new("main", "main");
        assert!(same.is_same());
        assert!(same.summary().contains("stay"));
    }
}
