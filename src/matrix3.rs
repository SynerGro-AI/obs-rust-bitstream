//! Safe pure Matrix3 (OBS graphics/matrix3.h core).
//!
//! The C version is a 3x4 matrix (x, y, z, t columns of Vec3).
//! The safe path is a Copy struct with identity and basic multiply.
//! Singular matrices are not inverted here; this is the pure foundation.
//! No unwraps.
//!
//! Differential: identity * m == m.

use crate::diff::assert_equal;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub t: Vec3,
}

impl Matrix3 {
    #[inline]
    pub fn identity() -> Self {
        Self {
            x: Vec3::new(1.0, 0.0, 0.0),
            y: Vec3::new(0.0, 1.0, 0.0),
            z: Vec3::new(0.0, 0.0, 1.0),
            t: Vec3::ZERO,
        }
    }

    #[inline]
    pub fn from_columns(x: Vec3, y: Vec3, z: Vec3, t: Vec3) -> Self {
        Self { x, y, z, t }
    }

    /// Multiply two 3x4 matrices (rotation/scale part + translation).
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        let x = self.transform_vec(other.x);
        let y = self.transform_vec(other.y);
        let z = self.transform_vec(other.z);
        let t = self.transform_point(other.t);
        Self { x, y, z, t }
    }

    #[inline]
    fn transform_vec(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.x.x * v.x + self.y.x * v.y + self.z.x * v.z,
            self.x.y * v.x + self.y.y * v.y + self.z.y * v.z,
            self.x.z * v.x + self.y.z * v.y + self.z.z * v.z,
        )
    }

    #[inline]
    fn transform_point(self, v: Vec3) -> Vec3 {
        self.transform_vec(v).add(self.t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let id = Matrix3::identity();
        assert_eq!(id.x.x, 1.0);
        assert_eq!(id.y.y, 1.0);
        assert_eq!(id.z.z, 1.0);
        assert_eq!(id.t, Vec3::ZERO);
    }

    #[test]
    fn mul_identity() {
        let m = Matrix3::from_columns(
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 0.0, 4.0),
            Vec3::new(1.0, 2.0, 3.0),
        );
        let out = m.mul(Matrix3::identity());
        assert_eq!(out, m);
    }

    #[test]
    fn differential() {
        let id = Matrix3::identity();
        let m = Matrix3::from_columns(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(5.0, 6.0, 7.0),
        );
        let out = id.mul(m);
        assert_equal(out.t.x, 5.0, "matrix3 mul identity");
    }
}
