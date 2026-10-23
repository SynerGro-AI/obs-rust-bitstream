//! Safe pure Quaternion (OBS graphics/quat.h core).
//!
//! The C version supports identity, mul, conjugate, and normalize.
//! The safe path is a Copy struct. Zero-length normalize returns None.
//! No unwraps.
//!
//! Differential: identity * q == q.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        )
    }

    #[inline]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    #[inline]
    pub fn normalize(self) -> Option<Self> {
        let len = self.length_sq().sqrt();
        if len <= 0.0 {
            None
        } else {
            Some(Self::new(
                self.x / len,
                self.y / len,
                self.z / len,
                self.w / len,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let q = Quat::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
    }

    #[test]
    fn mul_identity() {
        let q = Quat::new(0.0, 0.0, 0.0, 1.0);
        assert_eq!(q.mul(Quat::IDENTITY), q);
    }

    #[test]
    fn normalize_safe() {
        assert!(Quat::new(0.0, 0.0, 0.0, 0.0).normalize().is_none());
        let n = Quat::new(0.0, 0.0, 0.0, 2.0).normalize().unwrap();
        assert!((n.w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn differential() {
        let q = Quat::new(1.0, 0.0, 0.0, 0.0);
        let out = Quat::IDENTITY.mul(q);
        assert_equal(out.x, 1.0, "quat mul identity");
    }
}
