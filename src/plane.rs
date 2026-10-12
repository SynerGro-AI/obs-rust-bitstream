//! Safe pure Plane (OBS graphics/plane.h core).
//!
//! The C version is a normal (dir) plus distance.
//! The safe path is a Copy struct. Zero-length normal normalize
//! returns None. No unwraps.
//!
//! Differential: distance and normal match the set values.

use crate::diff::assert_equal;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub dir: Vec3,
    pub dist: f32,
}

impl Plane {
    #[inline]
    pub fn new(dir: Vec3, dist: f32) -> Self {
        Self { dir, dist }
    }

    #[inline]
    pub fn from_components(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            dir: Vec3::new(a, b, c),
            dist: d,
        }
    }

    /// Normalize the plane normal. Returns None if the normal length is zero.
    #[inline]
    pub fn normalize(self) -> Option<Self> {
        let n = self.dir.normalize()?;
        Some(Self {
            dir: n,
            dist: self.dist,
        })
    }

    /// Signed distance from a point to the plane.
    #[inline]
    pub fn distance_to_point(self, p: Vec3) -> f32 {
        self.dir.dot(p) - self.dist
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.dist == 0.0 && self.dir.length_sq() == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        let p = Plane::from_components(0.0, 1.0, 0.0, 5.0);
        assert_eq!(p.dir.y, 1.0);
        assert_eq!(p.dist, 5.0);
    }

    #[test]
    fn normalize_safe() {
        assert!(Plane::from_components(0.0, 0.0, 0.0, 1.0).normalize().is_none());
        let n = Plane::from_components(0.0, 2.0, 0.0, 1.0).normalize().unwrap();
        assert!((n.dir.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn distance() {
        let p = Plane::from_components(0.0, 1.0, 0.0, 0.0);
        assert_eq!(p.distance_to_point(Vec3::new(0.0, 3.0, 0.0)), 3.0);
    }

    #[test]
    fn differential() {
        let p = Plane::from_components(1.0, 0.0, 0.0, 2.0);
        assert_equal(p.dist, 2.0, "plane dist");
    }
}
