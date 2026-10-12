//! Safe pure CameraFollow (flight camera core).
//!
//! Positions a camera behind and above a bone so a flight can
//! be viewed. No unwraps.
//!
//! Differential: follow a bone at (0, 10) places the eye
//! behind and above it.

use crate::bone::Bone;
use crate::camera::Camera;
use crate::vec3::Vec3;

#[derive(Clone, Debug)]
pub struct CameraFollow {
    back: f32,
    up: f32,
}

impl CameraFollow {
    pub fn new(back: f32, up: f32) -> Self {
        Self { back, up }
    }

    /// Build a camera that looks at the bone from behind and above.
    pub fn follow(&self, bone: &Bone) -> Camera {
        let t = bone.local().t;
        Camera::new(
            Vec3::new(t.x, t.y + self.up, t.z + self.back),
            Vec3::new(t.x, t.y, t.z),
            Vec3::new(0.0, 1.0, 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_places_eye() {
        let mut bone = Bone::new("plane").unwrap();
        bone.set_local_translation(0.0, 10.0, 0.0);
        let f = CameraFollow::new(5.0, 2.0);
        let cam = f.follow(&bone);
        assert!((cam.eye.y - 12.0).abs() < 0.001);
        assert!((cam.eye.z - 5.0).abs() < 0.001);
        assert_eq!(cam.target.y, 10.0);
    }

    #[test]
    fn differential() {
        let mut bone = Bone::new("plane").unwrap();
        bone.set_local_translation(1.0, 2.0, 3.0);
        let f = CameraFollow::new(4.0, 1.0);
        let cam = f.follow(&bone);
        assert_eq!(cam.target.x, 1.0);
        assert_eq!(cam.eye.z, 7.0);
    }
}
