//! Safe pure TextTool (type tool).
//!
//! Holds a string, a position, a size, and a color.
//! Set the string, move it, or change the size.
//! Empty strings are rejected. No unwraps.
//!
//! Differential: a different string produces a different tool.

#[derive(Clone, Debug, PartialEq)]
pub struct TextTool {
    content: String,
    x: f32,
    y: f32,
    size: f32,
}

impl TextTool {
    /// Create a text tool. None if the content is empty.
    pub fn new(content: &str, x: f32, y: f32, size: f32) -> Option<Self> {
        if content.is_empty() {
            return None;
        }
        Some(Self {
            content: content.to_string(),
            x,
            y,
            size: size.max(1.0),
        })
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: &str) -> bool {
        if content.is_empty() {
            return false;
        }
        self.content = content.to_string();
        true
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.max(1.0);
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates() {
        let t = TextTool::new("Hello", 10.0, 20.0, 24.0).unwrap();
        assert_eq!(t.content(), "Hello");
        assert_eq!(t.position(), (10.0, 20.0));
        assert!((t.size() - 24.0).abs() < 0.01);
    }

    #[test]
    fn empty_rejected() {
        assert!(TextTool::new("", 0.0, 0.0, 12.0).is_none());
    }

    #[test]
    fn differential() {
        let a = TextTool::new("A", 0.0, 0.0, 12.0).unwrap();
        let b = TextTool::new("B", 0.0, 0.0, 12.0).unwrap();
        assert_ne!(a.content(), b.content());
    }
}
