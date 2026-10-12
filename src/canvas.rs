//! Safe pure Canvas (OBS render surface core).
//!
//! A canvas owns a name, a size, and an optional scene name it
//! is rendering. Empty names are rejected. No unwraps.
//!
//! Differential: set scene + get round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Canvas {
    name: String,
    width: u32,
    height: u32,
    scene: Option<String>,
}

impl Canvas {
    pub fn new(name: &str, width: u32, height: u32) -> Result<Self, CanvasError> {
        if name.is_empty() {
            return Err(CanvasError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            width,
            height,
            scene: None,
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

    pub fn scene(&self) -> Option<&str> {
        self.scene.as_deref()
    }

    pub fn set_scene(&mut self, name: &str) {
        self.scene = Some(name.to_string());
    }

    pub fn clear_scene(&mut self) {
        self.scene = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = Canvas::new("preview", 1280, 720).unwrap();
        assert_eq!(c.size(), (1280, 720));
        assert!(c.scene().is_none());
    }

    #[test]
    fn empty_name() {
        assert!(Canvas::new("", 1, 1).is_err());
    }

    #[test]
    fn differential() {
        let mut c = Canvas::new("program", 1920, 1080).unwrap();
        c.set_scene("Main");
        assert_eq!(c.scene(), Some("Main"));
        c.clear_scene();
        assert!(c.scene().is_none());
    }
}
