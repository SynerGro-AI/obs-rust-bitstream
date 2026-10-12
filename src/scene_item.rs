//! Safe pure SceneItem (OBS scene item core).
//!
//! An item owns an id, a source name, visibility, and a 2D transform
//! (position, scale, rotation). Empty ids are rejected. No unwraps.
//!
//! Differential: set transform + get round-trips.

use crate::vec2::Vec2;

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub pos: Vec2,
    pub scale: Vec2,
    pub rot: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            rot: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneItemError {
    EmptyId,
    EmptySource,
}

#[derive(Clone, Debug)]
pub struct SceneItem {
    id: String,
    source: String,
    visible: bool,
    transform: Transform,
}

impl SceneItem {
    pub fn new(id: &str, source: &str) -> Result<Self, SceneItemError> {
        if id.is_empty() {
            return Err(SceneItemError::EmptyId);
        }
        if source.is_empty() {
            return Err(SceneItemError::EmptySource);
        }
        Ok(Self {
            id: id.to_string(),
            source: source.to_string(),
            visible: true,
            transform: Transform::default(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, v: bool) {
        self.visible = v;
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.transform.pos = Vec2::new(x, y);
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.transform.scale = Vec2::new(x, y);
    }

    pub fn set_rot(&mut self, deg: f32) {
        self.transform.rot = deg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let i = SceneItem::new("i1", "cam").unwrap();
        assert_eq!(i.id(), "i1");
        assert_eq!(i.source(), "cam");
        assert!(i.visible());
    }

    #[test]
    fn empty_rejected() {
        assert!(SceneItem::new("", "cam").is_err());
        assert!(SceneItem::new("i", "").is_err());
    }

    #[test]
    fn transform_round_trip() {
        let mut i = SceneItem::new("i", "src").unwrap();
        i.set_pos(100.0, 200.0);
        i.set_scale(2.0, 2.0);
        i.set_rot(45.0);
        i.set_visible(false);
        assert_eq!(i.transform().pos.x, 100.0);
        assert_eq!(i.transform().scale.y, 2.0);
        assert_eq!(i.transform().rot, 45.0);
        assert!(!i.visible());
    }

    #[test]
    fn differential() {
        let mut i = SceneItem::new("item", "overlay").unwrap();
        i.set_pos(10.0, 20.0);
        assert_eq!(i.transform().pos.y, 20.0);
    }
}
