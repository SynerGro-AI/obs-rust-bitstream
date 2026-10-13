//! Safe pure TopicBackground (live background core).
//!
//! Renders a background color from a topic. No unwraps.
//!
//! Differential: weather paints blue, sports paints green.

#[derive(Clone, Debug)]
pub struct TopicBackground;

impl TopicBackground {
    /// Background color for a topic.
    pub fn render(topic: &str) -> (f32, f32, f32) {
        let t = topic.to_lowercase();
        if t.contains("weather") {
            (0.2, 0.5, 0.9)
        } else if t.contains("sport") {
            (0.1, 0.7, 0.2)
        } else if t.contains("breaking") {
            (0.8, 0.1, 0.1)
        } else {
            (0.15, 0.15, 0.2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather() {
        let (r, g, b) = TopicBackground::render("weather");
        assert!(b > r);
    }

    #[test]
    fn differential() {
        let w = TopicBackground::render("weather");
        let s = TopicBackground::render("sports");
        assert_ne!(w, s);
    }
}
