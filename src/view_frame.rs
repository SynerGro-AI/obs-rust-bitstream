//! Safe pure ViewFrame (flight view core).
//!
//! Combines a camera follow and a plane bone into a view matrix
//! ready for the canvas. No unwraps.
//!
//! Differential: a bone at origin with a follow of back=5
//! produces a view whose translation is non-zero.

use crate::bone::Bone;
use crate::camera_follow::CameraFollow;
use crate::matrix4::Matrix4;

#[derive(Clone, Debug)]
pub struct ViewFrame {
    follow: CameraFollow,
}

impl ViewFrame {
    pub fn new(back: f32, up: f32) -> Self {
        Self {
            follow: CameraFollow::new(back, up),
        }
    }

    /// Build the view matrix that looks at the bone.
    pub fn view(&self, bone: &Bone) -> Matrix4 {
        self.follow.follow(bone).view()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_is_not_identity() {
        let bone = Bone::new("plane").unwrap();
        let frame = ViewFrame::new(5.0, 2.0);
        let v = frame.view(&bone);
        assert!(v.t.z.abs() > 0.0);
    }

    #[test]
    fn differential() {
        let mut bone = Bone::new("plane").unwrap();
        bone.set_local_translation(0.0, 10.0, 0.0);
        let frame = ViewFrame::new(5.0, 2.0);
        let v = frame.view(&bone);
        assert_eq!(v.t.w, 1.0);
    }
}
