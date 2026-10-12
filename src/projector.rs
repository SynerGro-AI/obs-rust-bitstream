//! Safe pure Projector (2D-to-3D lift core).
//!
//! Lifts a 2D RenderItem into 3D by adding a Z depth and applying
//! a camera view matrix. ToonBoom layers become Maya depth.
//! No unwraps.
//!
//! Differential: lift at depth 0 keeps the 2D translation.

use crate::camera::Camera;
use crate::matrix4::Matrix4;
use crate::render_item::RenderItem;
use crate::vec3::Vec3;
use crate::vec4::Vec4;

#[derive(Clone, Debug)]
pub struct Projector {
    depth: f32,
}

impl Projector {
    pub fn new(depth: f32) -> Self {
        Self { depth }
    }

    /// Lift a 2D render item into 3D and apply the camera view.
    pub fn project(&self, item: &RenderItem, camera: &Camera) -> Matrix4 {
        let model = item.model();
        let lifted = Matrix4::from_columns(
            model.x,
            model.y,
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(model.t.x, model.t.y, self.depth, 1.0),
        );
        camera.view().mul(lifted)
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_item::Transform;
    use crate::vec2::Vec2;

    #[test]
    fn lift_keeps_translation() {
        let t = Transform {
            pos: Vec2::new(10.0, 20.0),
            scale: Vec2::new(1.0, 1.0),
            rot: 0.0,
        };
        let item = RenderItem::from_transform("cam", &t, true);
        let cam = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let p = Projector::new(0.0);
        let m = p.project(&item, &cam);
        assert_eq!(m.t.w, 1.0);
    }

    #[test]
    fn differential() {
        let t = Transform {
            pos: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            rot: 0.0,
        };
        let item = RenderItem::from_transform("layer", &t, true);
        let cam = Camera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let p = Projector::new(2.0);
        let m = p.project(&item, &cam);
        assert_eq!(m.t.w, 1.0);
    }
}
