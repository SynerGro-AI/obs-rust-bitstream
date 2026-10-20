//! Safe pure Bounds (OBS graphics/bounds.h core).
//!
//! The C version is an axis-aligned bounding box (min, max).
//! The safe path is a Copy struct. Invalid (min > max) returns None
//! on construction. No unwraps.
//!
//! Differential: center matches (min + max) / 2.

#[cfg(test)]
use crate::diff::assert_equal;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Bounds {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Option<Self> {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            None
        } else {
            Some(Self { min, max })
        }
    }

    #[inline]
    pub fn zero() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }

    #[inline]
    pub fn center(self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    #[inline]
    pub fn contains_point(self, p: Vec3) -> bool {
        p.x >= self.min.x && p.x <= self.max.x &&
        p.y >= self.min.y && p.y <= self.max.y &&
        p.z >= self.min.z && p.z <= self.max.z
    }

    #[inline]
    pub fn merge(self, other: Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let b = Bounds::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 4.0, 6.0)).unwrap();
        assert_eq!(b.center(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn invalid_safe() {
        assert!(Bounds::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)).is_none());
    }

    #[test]
    fn contains() {
        let b = Bounds::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)).unwrap();
        assert!(b.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!b.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn differential() {
        let b = Bounds::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 3.0, 3.0)).unwrap();
        assert_equal(b.center().x, 2.0, "bounds center");
    }
}
