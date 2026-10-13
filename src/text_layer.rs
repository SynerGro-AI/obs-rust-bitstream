//! Safe pure TextLayer (text core).
//!
//! A text layer owns a name, a string, a size, and a color.
//! Empty names rejected. No unwraps.
//!
//! Differential: set text + size round-trips.

use crate::vec3::Vec3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct TextLayer {
    name: String,
    text: String,
    size: f32,
    color: Vec3,
}

impl TextLayer {
    pub fn new(name: &str) -> Result<Self, TextError> {
        if name.is_empty() {
            return Err(TextError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            text: String::new(),
            size: 24.0,
            color: Vec3::new(1.0, 1.0, 1.0),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_text(&mut self, t: &str) {
        self.text = t.to_string();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_size(&mut self, s: f32) {
        self.size = s.max(1.0);
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_color(&mut self, c: Vec3) {
        self.color = c;
    }

    pub fn color(&self) -> Vec3 {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = TextLayer::new("title").unwrap();
        assert_eq!(t.size(), 24.0);
    }

    #[test]
    fn empty_name() {
        assert!(TextLayer::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut t = TextLayer::new("title").unwrap();
        t.set_text("Hello");
        t.set_size(48.0);
        assert_eq!(t.text(), "Hello");
        assert_eq!(t.size(), 48.0);
    }
}
