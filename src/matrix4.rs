//! Safe pure Matrix4 (OBS graphics/matrix4.h core).
//!
//! The C version is a 4x4 column-major matrix (x, y, z, t columns of Vec4).
//! The safe path is a Copy struct with identity and affine multiply.
//! No unwraps. Full general multiply can be expanded later.
//!
//! Differential: identity * m == m.

#[cfg(test)]
use crate::diff::assert_equal;
use crate::vec4::Vec4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub t: Vec4,
}

impl Matrix4 {
    #[inline]
    pub fn identity() -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            t: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    #[inline]
    pub fn from_columns(x: Vec4, y: Vec4, z: Vec4, t: Vec4) -> Self {
        Self { x, y, z, t }
    }

    /// Affine multiply (rotation/scale columns + translation).
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        let x = self.transform_vec(other.x);
        let y = self.transform_vec(other.y);
        let z = self.transform_vec(other.z);
        let t = self.transform_point(other.t);
        Self { x, y, z, t }
    }

    #[inline]
    fn transform_vec(self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.x.x * v.x + self.y.x * v.y + self.z.x * v.z,
            self.x.y * v.x + self.y.y * v.y + self.z.y * v.z,
            self.x.z * v.x + self.y.z * v.y + self.z.z * v.z,
            0.0,
        )
    }

    #[inline]
    fn transform_point(self, v: Vec4) -> Vec4 {
        let r = self.transform_vec(v);
        Vec4::new(r.x + self.t.x, r.y + self.t.y, r.z + self.t.z, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let id = Matrix4::identity();
        assert_eq!(id.x.x, 1.0);
        assert_eq!(id.y.y, 1.0);
        assert_eq!(id.z.z, 1.0);
        assert_eq!(id.t.w, 1.0);
    }

    #[test]
    fn mul_identity() {
        let m = Matrix4::from_columns(
            Vec4::new(2.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 3.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 4.0, 0.0),
            Vec4::new(1.0, 2.0, 3.0, 1.0),
        );
        let out = m.mul(Matrix4::identity());
        assert_eq!(out, m);
    }

    #[test]
    fn differential() {
        let id = Matrix4::identity();
        let m = Matrix4::from_columns(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(5.0, 6.0, 7.0, 1.0),
        );
        let out = id.mul(m);
        assert_equal(out.t.x, 5.0, "matrix4 mul identity");
    }
}
