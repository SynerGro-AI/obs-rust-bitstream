//! Safe pure DisplayContext (OBS display / preview core).
//!
//! A display owns a name, a size, and an optional source name
//! it is rendering. Empty names are rejected. No unwraps.
//!
//! Differential: set size + get round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct DisplayContext {
    name: String,
    width: u32,
    height: u32,
    source: Option<String>,
}

impl DisplayContext {
    pub fn new(name: &str, width: u32, height: u32) -> Result<Self, DisplayError> {
        if name.is_empty() {
            return Err(DisplayError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            width,
            height,
            source: None,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn set_source(&mut self, name: &str) {
        self.source = Some(name.to_string());
    }

    pub fn clear_source(&mut self) {
        self.source = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let d = DisplayContext::new("preview", 1280, 720).unwrap();
        assert_eq!(d.name(), "preview");
        assert_eq!(d.size(), (1280, 720));
        assert!(d.source().is_none());
    }

    #[test]
    fn empty_name() {
        assert!(DisplayContext::new("", 1, 1).is_err());
    }

    #[test]
    fn differential() {
        let mut d = DisplayContext::new("program", 1920, 1080).unwrap();
        d.set_size(1280, 720);
        d.set_source("Main");
        assert_eq!(d.size(), (1280, 720));
        assert_eq!(d.source(), Some("Main"));
        d.clear_source();
        assert!(d.source().is_none());
    }
}
