//! Safe pure video scaler helpers (OBS video-scaler.h core).
//!
//! The C version wraps FFmpeg swscale. The safe path provides a pure
//! ScaleInfo and a nearest-neighbor output size calculator that never
//! overflows and returns None on zero dimensions. No external deps.
//!
//! Differential: output size matches the scale factor.

use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaleInfo {
    pub width: u32,
    pub height: u32,
}

impl ScaleInfo {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self { width, height })
        }
    }

    /// Pure nearest-neighbor output size for a scale factor (integer).
    pub fn scaled(self, factor: u32) -> Option<Self> {
        if factor == 0 {
            return None;
        }
        let w = self.width.checked_mul(factor)?;
        let h = self.height.checked_mul(factor)?;
        Self::new(w, h)
    }
}

/// Simple bilinear weight (0..1) for a pure interpolation foundation.
pub fn lerp_weight(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let s = ScaleInfo::new(1920, 1080).unwrap();
        assert_eq!(s.width, 1920);
    }

    #[test]
    fn scaled() {
        let s = ScaleInfo::new(4, 2).unwrap();
        let out = s.scaled(2).unwrap();
        assert_eq!(out.width, 8);
        assert_eq!(out.height, 4);
    }

    #[test]
    fn zero_safe() {
        assert!(ScaleInfo::new(0, 10).is_none());
        let s = ScaleInfo::new(10, 10).unwrap();
        assert!(s.scaled(0).is_none());
    }

    #[test]
    fn differential() {
        let s = ScaleInfo::new(3, 3).unwrap();
        let out = s.scaled(3).unwrap();
        assert_equal(out.width, 9, "scaled width");
    }
}
