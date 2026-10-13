//! Safe pure CueSheet (show rundown core).
//!
//! Holds an ordered list of topics. next advances. No unwraps.
//!
//! Differential: three cues + next round-trips.

#[derive(Clone, Debug, Default)]
pub struct CueSheet {
    cues: Vec<String>,
    index: usize,
}

impl CueSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, topic: &str) {
        if !topic.is_empty() {
            self.cues.push(topic.to_string());
        }
    }

    pub fn len(&self) -> usize {
        self.cues.len()
    }

    pub fn current(&self) -> Option<&str> {
        self.cues.get(self.index).map(|s| s.as_str())
    }

    /// Advance to the next cue. Returns the new current topic.
    pub fn next(&mut self) -> Option<&str> {
        if self.index + 1 < self.cues.len() {
            self.index += 1;
        }
        self.current()
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = CueSheet::new();
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn differential() {
        let mut c = CueSheet::new();
        c.add("weather");
        c.add("sports");
        c.add("closing");
        assert_eq!(c.current(), Some("weather"));
        assert_eq!(c.next(), Some("sports"));
        assert_eq!(c.next(), Some("closing"));
        assert_eq!(c.next(), Some("closing"));
    }
}
