//! Safe pure AxisAng (OBS graphics/axisang.h core).
//!
//! The C version is an axis (x,y,z) plus angle (w) in radians.
//! The safe path is a Copy struct. Zero-length axis normalize
//! returns None. No unwraps.
//!
//! Differential: set and identity-style zero match.

use crate::diff::assert_equal;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisAng {
    pub axis: Vec3,
    pub angle: f32,
}

impl AxisAng {
    pub const ZERO: Self = Self {
        axis: Vec3::ZERO,
        angle: 0.0,
    };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, angle: f32) -> Self {
        Self {
            axis: Vec3::new(x, y, z),
            angle,
        }
    }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        Self { axis, angle }
    }

    /// Normalize the axis. Returns None if the axis length is zero.
    #[inline]
    pub fn normalize_axis(self) -> Option<Self> {
        let n = self.axis.normalize()?;
        Some(Self {
            axis: n,
            angle: self.angle,
        })
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.angle == 0.0 && self.axis.length_sq() == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        let aa = AxisAng::new(0.0, 1.0, 0.0, 1.57);
        assert_eq!(aa.axis.y, 1.0);
        assert!((aa.angle - 1.57).abs() < 1e-5);
    }

    #[test]
    fn normalize_safe() {
        assert!(AxisAng::ZERO.normalize_axis().is_none());
        let n = AxisAng::new(0.0, 2.0, 0.0, 0.5).normalize_axis().unwrap();
        assert!((n.axis.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn zero() {
        assert!(AxisAng::ZERO.is_zero());
    }

    #[test]
    fn differential() {
        let aa = AxisAng::new(1.0, 0.0, 0.0, 0.0);
        assert_equal(aa.angle, 0.0, "axisang angle");
    }
}
