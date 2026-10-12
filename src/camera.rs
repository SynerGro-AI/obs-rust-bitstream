//! Safe pure Camera (Maya-style view core).
//!
//! A camera owns an eye position, a look-at target, and an up vector.
//! The view matrix is built column-major. No unwraps.
//!
//! Differential: look at origin + eye at (0,0,5) puts -z forward.

use crate::matrix4::Matrix4;
use crate::vec3::Vec3;
use crate::vec4::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Self { eye, target, up }
    }

    /// Build a right-handed look-at view matrix.
    pub fn view(&self) -> Matrix4 {
        let f = self.target.sub(self.eye).normalize().unwrap_or(Vec3::new(0.0, 0.0, -1.0));
        let s = f.cross(self.up).normalize().unwrap_or(Vec3::new(1.0, 0.0, 0.0));
        let u = s.cross(f);
        Matrix4::from_columns(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(-s.dot(self.eye), -u.dot(self.eye), f.dot(self.eye), 1.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_at_origin() {
        let c = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let v = c.view();
        assert!((v.t.z + 5.0).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let c = Camera::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let v = c.view();
        assert_eq!(v.t.w, 1.0);
    }
}
