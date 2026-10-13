//! Safe pure SmartObject (non-destructive layer core).
//!
//! A smart object owns a name and a source. It can be
//! transformed without changing the source. Empty names
//! rejected. No unwraps.
//!
//! Differential: set source + transform round-trips.

use crate::transform::Transform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SmartError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct SmartObject {
    name: String,
    source: String,
    transform: Transform,
}

impl SmartObject {
    pub fn new(name: &str) -> Result<Self, SmartError> {
        if name.is_empty() {
            return Err(SmartError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            source: String::new(),
            transform: Transform::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_source(&mut self, src: &str) {
        self.source = src.to_string();
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = SmartObject::new("logo").unwrap();
        assert!(s.source().is_empty());
    }

    #[test]
    fn empty_name() {
        assert!(SmartObject::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = SmartObject::new("logo").unwrap();
        s.set_source("logo.png");
        s.transform_mut().set_scale(2.0);
        assert_eq!(s.source(), "logo.png");
        assert_eq!(s.transform().scale(), 2.0);
    }
}
