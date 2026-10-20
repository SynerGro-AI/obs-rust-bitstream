//! Safe pure Vec3 (OBS graphics/vec3.h core).
//!
//! The C version is a 3-float vector (with SIMD w=0) supporting
//! add/sub/mul/div/dot/cross/length. The safe path is a Copy struct.
//! Division by zero and zero-length normalize return None / zero.
//! No unwraps.
//!
//! Differential: add, dot, and cross match the mathematical definitions.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    #[inline]
    pub fn div(self, other: Self) -> Option<Self> {
        if other.x == 0.0 || other.y == 0.0 || other.z == 0.0 {
            None
        } else {
            Some(Self::new(self.x / other.x, self.y / other.y, self.z / other.z))
        }
    }

    /// Fast path for graphics: assumes non-zero divisor.
    #[inline]
    pub fn div_fast(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }

    #[inline]
    pub fn mulf(self, f: f32) -> Self {
        Self::new(self.x * f, self.y * f, self.z * f)
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        let sq = self.length_sq();
        if sq <= 0.0 {
            0.0
        } else {
            sq.sqrt()
        }
    }

    #[inline]
    pub fn normalize(self) -> Option<Self> {
        let len = self.length();
        if len <= 0.0 {
            None
        } else {
            Some(self.mulf(1.0 / len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.add(b), Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b.sub(a), Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn cross() {
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(i.cross(j), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn div_normalize_safe() {
        let a = Vec3::new(2.0, 4.0, 6.0);
        assert!(a.div(Vec3::ZERO).is_none());
        assert!(Vec3::ZERO.normalize().is_none());
        let n = Vec3::new(3.0, 0.0, 0.0).normalize().unwrap();
        assert!((n.x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn differential() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_equal(a.dot(b), 32.0, "vec3 dot");
    }
}
