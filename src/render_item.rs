//! Safe pure RenderItem (OBS draw command core).
//!
//! A render item owns a source name and a model matrix built from
//! a scene item's transform. No unwraps.
//!
//! Differential: build from transform + read translation round-trips.

use crate::matrix4::Matrix4;
use crate::scene_item::Transform;
use crate::vec4::Vec4;

#[derive(Clone, Debug)]
pub struct RenderItem {
    source: String,
    model: Matrix4,
    visible: bool,
}

impl RenderItem {
    /// Build a render item from a source name and a scene transform.
    /// The model is scale * rotation * translation, column-major.
    pub fn from_transform(source: &str, t: &Transform, visible: bool) -> Self {
        let rad = t.rot.to_radians();
        let (s, c) = rad.sin_cos();
        let sx = t.scale.x;
        let sy = t.scale.y;
        let model = Matrix4::from_columns(
            Vec4::new(c * sx, s * sx, 0.0, 0.0),
            Vec4::new(-s * sy, c * sy, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(t.pos.x, t.pos.y, 0.0, 1.0),
        );
        Self {
            source: source.to_string(),
            model,
            visible,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn model(&self) -> &Matrix4 {
        &self.model
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Translation extracted from the model matrix (t column).
    pub fn translation(&self) -> (f32, f32) {
        (self.model.t.x, self.model.t.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec2::Vec2;

    #[test]
    fn from_transform() {
        let t = Transform {
            pos: Vec2::new(100.0, 200.0),
            scale: Vec2::new(1.0, 1.0),
            rot: 0.0,
        };
        let r = RenderItem::from_transform("cam", &t, true);
        assert_eq!(r.source(), "cam");
        assert!(r.visible());
        assert_eq!(r.translation(), (100.0, 200.0));
    }

    #[test]
    fn differential() {
        let t = Transform {
            pos: Vec2::new(50.0, 60.0),
            scale: Vec2::new(2.0, 2.0),
            rot: 0.0,
        };
        let r = RenderItem::from_transform("overlay", &t, false);
        assert!(!r.visible());
        assert_eq!(r.translation(), (50.0, 60.0));
        assert_eq!(r.model().x.x, 2.0);
    }
}
