//! Safe pure LowerThird (on-screen topic core).
//!
//! Renders the current topic as an on-screen graphic.
//! No unwraps.
//!
//! Differential: a topic produces a non-empty label.

#[derive(Clone, Debug)]
pub struct LowerThird {
    text: String,
    visible: bool,
}

impl LowerThird {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            visible: false,
        }
    }

    pub fn set(&mut self, topic: &str) {
        self.text = topic.to_string();
        self.visible = !topic.is_empty();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Label shown on screen. Empty when hidden.
    pub fn label(&self) -> &str {
        if self.visible { &self.text } else { "" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let l = LowerThird::new();
        assert!(!l.visible());
    }

    #[test]
    fn differential() {
        let mut l = LowerThird::new();
        l.set("weather");
        assert!(l.visible());
        assert_eq!(l.label(), "weather");
        l.hide();
        assert_eq!(l.label(), "");
    }
}
