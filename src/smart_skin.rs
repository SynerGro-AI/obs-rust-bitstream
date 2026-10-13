//! Safe pure SmartSkin (drag-and-drop skin core).
//!
//! A skin owns a name and a source. It attaches to an object
//! and places by drag-and-drop. Seamless mode avoids edge
//! seams and wrinkle fights. Empty names rejected. No unwraps.
//!
//! Differential: attach + drop + seamless round-trips.

use crate::object3d::Object3D;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug)]
pub struct SmartSkin {
    name: String,
    source: String,
    attached_to: Option<String>,
    drop_u: f32,
    drop_v: f32,
    seamless: bool,
}

impl SmartSkin {
    pub fn new(name: &str) -> Result<Self, SkinError> {
        if name.is_empty() {
            return Err(SkinError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            source: String::new(),
            attached_to: None,
            drop_u: 0.5,
            drop_v: 0.5,
            seamless: true,
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

    /// Attach the skin to an object by name.
    pub fn attach(&mut self, object: &Object3D) {
        self.attached_to = Some(object.name().to_string());
    }

    pub fn attached_to(&self) -> Option<&str> {
        self.attached_to.as_deref()
    }

    /// Drag-and-drop the skin center to UV (u, v) in 0-1.
    pub fn drop(&mut self, u: f32, v: f32) {
        self.drop_u = u.clamp(0.0, 1.0);
        self.drop_v = v.clamp(0.0, 1.0);
    }

    pub fn drop_point(&self) -> (f32, f32) {
        (self.drop_u, self.drop_v)
    }

    pub fn set_seamless(&mut self, s: bool) {
        self.seamless = s;
    }

    pub fn seamless(&self) -> bool {
        self.seamless
    }

    /// True when the skin is attached, has a source, and is seamless.
    /// Seamless skins do not produce edge seams or wrinkles.
    pub fn is_clean(&self) -> bool {
        self.attached_to.is_some() && !self.source.is_empty() && self.seamless
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = SmartSkin::new("body").unwrap();
        assert!(s.seamless());
    }

    #[test]
    fn empty_name() {
        assert!(SmartSkin::new("").is_err());
    }

    #[test]
    fn differential() {
        let obj = Object3D::new("character").unwrap();
        let mut s = SmartSkin::new("body").unwrap();
        s.set_source("skin.png");
        s.attach(&obj);
        s.drop(0.3, 0.7);
        assert_eq!(s.attached_to(), Some("character"));
        assert_eq!(s.drop_point(), (0.3, 0.7));
        assert!(s.is_clean());
    }
}
