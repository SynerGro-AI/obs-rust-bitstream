//! Safe pure Speech (live speech core).
//!
//! Holds the last line heard. The director reads it.
//! No unwraps.
//!
//! Differential: hear a line + last round-trips.

#[derive(Clone, Debug, Default)]
pub struct Speech {
    last: String,
}

impl Speech {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hear(&mut self, line: &str) {
        self.last = line.to_string();
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
        let s = Speech::new();
        assert!(s.is_empty());
    }

    #[test]
    fn differential() {
        let mut s = Speech::new();
        s.hear("sunset");
        assert_eq!(s.last(), "sunset");
        s.clear();
        assert!(s.is_empty());
    }
}
