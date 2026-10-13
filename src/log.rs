//! Safe pure Log (action history core).
//!
//! Records what the root did. No unwraps.
//!
//! Differential: two entries round-trip.

#[derive(Clone, Debug, Default)]
pub struct Log {
    entries: Vec<String>,
}

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: &str) {
        if !entry.is_empty() {
            self.entries.push(entry.to_string());
        }
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn last(&self) -> Option<&str> {
        self.entries.last().map(|s| s.as_str())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let l = Log::new();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn differential() {
        let mut l = Log::new();
        l.add("press r");
        l.add("start recording");
        assert_eq!(l.len(), 2);
        assert_eq!(l.last(), Some("start recording"));
    }
}
