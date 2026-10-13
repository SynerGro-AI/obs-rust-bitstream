//! Safe pure ShowState (live state core).
//!
//! Tracks whether the show is off, in preview, or live.
//! No unwraps.
//!
//! Differential: off -> preview -> live round-trips.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShowMode {
    #[default]
    Off,
    Preview,
    Live,
}

#[derive(Clone, Debug, Default)]
pub struct ShowState {
    mode: ShowMode,
    topic: String,
}

impl ShowState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> ShowMode {
        self.mode
    }

    pub fn preview(&mut self) {
        self.mode = ShowMode::Preview;
    }

    pub fn go_live(&mut self) {
        self.mode = ShowMode::Live;
    }

    pub fn stop(&mut self) {
        self.mode = ShowMode::Off;
    }

    pub fn is_live(&self) -> bool {
        self.mode == ShowMode::Live
    }

    pub fn set_topic(&mut self, topic: &str) {
        self.topic = topic.to_string();
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = ShowState::new();
        assert_eq!(s.mode(), ShowMode::Off);
    }

    #[test]
    fn differential() {
        let mut s = ShowState::new();
        s.preview();
        assert_eq!(s.mode(), ShowMode::Preview);
        s.set_topic("concert");
        s.go_live();
        assert!(s.is_live());
        assert_eq!(s.topic(), "concert");
        s.stop();
        assert!(!s.is_live());
    }
}
