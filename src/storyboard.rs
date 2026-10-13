//! Safe pure Storyboard (planned story core).
//!
//! Holds ordered beats. Each beat has a line the director
//! can react to. No unwraps.
//!
//! Differential: three beats + current + next round-trips.

#[derive(Clone, Debug)]
pub struct Beat {
    pub line: String,
}

#[derive(Clone, Debug, Default)]
pub struct Storyboard {
    beats: Vec<Beat>,
    index: usize,
}

impl Storyboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, line: &str) {
        if !line.is_empty() {
            self.beats.push(Beat {
                line: line.to_string(),
            });
        }
    }

    pub fn len(&self) -> usize {
        self.beats.len()
    }

    pub fn current(&self) -> Option<&str> {
        self.beats.get(self.index).map(|b| b.line.as_str())
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.index + 1 < self.beats.len() {
            self.index += 1;
        }
        self.current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = Storyboard::new();
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn differential() {
        let mut s = Storyboard::new();
        s.add("weather in Calgary");
        s.add("sunset");
        s.add("closing");
        assert_eq!(s.current(), Some("weather in Calgary"));
        assert_eq!(s.next(), Some("sunset"));
    }
}
