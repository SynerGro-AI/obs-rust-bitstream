//! Safe pure Vec4 (OBS graphics/vec4.h core).
//!
//! The C version is a 4-float vector with add/sub/mul/div/dot.
//! The safe path is a Copy struct. Division by zero returns None.
//! No unwraps.
//!
//! Differential: add and dot match the mathematical definitions.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z, self.w - other.w)
    }

    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z, self.w * other.w)
    }

    #[inline]
    pub fn div(self, other: Self) -> Option<Self> {
        if other.x == 0.0 || other.y == 0.0 || other.z == 0.0 || other.w == 0.0 {
            None
        } else {
            Some(Self::new(self.x / other.x, self.y / other.y, self.z / other.z, self.w / other.w))
        }
    }

    /// Fast path for graphics: assumes non-zero divisor.
    #[inline]
    pub fn div_fast(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z, self.w / other.w)
    }

    #[inline]
    pub fn mulf(self, f: f32) -> Self {
        Self::new(self.x * f, self.y * f, self.z * f, self.w * f)
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4::new(5.0, 6.0, 7.0, 8.0);
        assert_eq!(a.add(b), Vec4::new(6.0, 8.0, 10.0, 12.0));
        assert_eq!(b.sub(a), Vec4::new(4.0, 4.0, 4.0, 4.0));
    }

    #[test]
    fn div_safe() {
        let a = Vec4::new(2.0, 4.0, 6.0, 8.0);
        assert!(a.div(Vec4::ZERO).is_none());
        assert_eq!(a.div(Vec4::new(2.0, 2.0, 2.0, 2.0)), Some(Vec4::new(1.0, 2.0, 3.0, 4.0)));
    }

    #[test]
    fn dot() {
        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(a.dot(a), 30.0);
    }

    #[test]
    fn differential() {
        let a = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let b = Vec4::new(2.0, 3.0, 4.0, 5.0);
        assert_equal(a.add(b).x, 3.0, "vec4 add");
    }
}
