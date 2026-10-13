//! Safe pure WindowCapture (window name holder core).
//!
//! Holds a window title and a cursor flag. Empty titles
//! rejected. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowError {
    EmptyTitle,
}

#[derive(Clone, Debug)]
pub struct WindowCapture {
    title: String,
    cursor: bool,
}

impl WindowCapture {
    pub fn new(title: &str, cursor: bool) -> Result<Self, WindowError> {
        if title.is_empty() {
            return Err(WindowError::EmptyTitle);
        }
        Ok(Self {
            title: title.to_string(),
            cursor,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn cursor(&self) -> bool {
        self.cursor
    }

    pub fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        if title.is_empty() {
            return Err(WindowError::EmptyTitle);
        }
        self.title = title.to_string();
        Ok(())
    }

    pub fn set_cursor(&mut self, cursor: bool) {
        self.cursor = cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(WindowCapture::new("", true).is_err());
    }

    #[test]
    fn differential() {
        let mut w = WindowCapture::new("OBS", false).unwrap();
        assert_eq!(w.title(), "OBS");
        assert!(!w.cursor());
        w.set_title("Chrome").unwrap();
        w.set_cursor(true);
        assert_eq!(w.title(), "Chrome");
        assert!(w.cursor());
    }
}
