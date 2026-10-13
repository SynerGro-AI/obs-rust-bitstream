//! Safe pure TextSource (text overlay core).
//!
//! Holds a string and a font size. Empty text rejected.
//! No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextError {
    Empty,
}

#[derive(Clone, Debug)]
pub struct TextSource {
    text: String,
    size: u32,
}

impl TextSource {
    pub fn new(text: &str, size: u32) -> Result<Self, TextError> {
        if text.is_empty() || size == 0 {
            return Err(TextError::Empty);
        }
        Ok(Self {
            text: text.to_string(),
            size,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_text(&mut self, text: &str) -> Result<(), TextError> {
        if text.is_empty() {
            return Err(TextError::Empty);
        }
        self.text = text.to_string();
        Ok(())
    }

    pub fn set_size(&mut self, size: u32) -> Result<(), TextError> {
        if size == 0 {
            return Err(TextError::Empty);
        }
        self.size = size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(TextSource::new("", 16).is_err());
        assert!(TextSource::new("hi", 0).is_err());
    }

    #[test]
    fn differential() {
        let mut t = TextSource::new("hello", 24).unwrap();
        assert_eq!(t.text(), "hello");
        assert_eq!(t.size(), 24);
        t.set_text("world").unwrap();
        t.set_size(32).unwrap();
        assert_eq!(t.text(), "world");
        assert_eq!(t.size(), 32);
    }
}
