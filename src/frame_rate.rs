//! Safe pure frame-rate helpers (OBS frame-rate.h).
//!
//! The C version stores a rational FPS and converts to interval or
//! floating-point rate. The safe path uses a simple struct and
//! returns Option on invalid (zero num or den). No unwraps.
//!
//! Differential: conversions match the mathematical definitions.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaFps {
    pub numerator: u32,
    pub denominator: u32,
}

impl MediaFps {
    pub fn new(numerator: u32, denominator: u32) -> Option<Self> {
        if numerator == 0 || denominator == 0 {
            None
        } else {
            Some(Self { numerator, denominator })
        }
    }

    pub fn is_valid(self) -> bool {
        self.numerator != 0 && self.denominator != 0
    }

    pub fn to_fps(self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        self.numerator as f64 / self.denominator as f64
    }

    pub fn to_frame_interval(self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        self.denominator as f64 / self.numerator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let fps = MediaFps::new(30000, 1001).unwrap();
        assert!(fps.is_valid());
        assert!((fps.to_fps() - 29.970).abs() < 0.01);
    }

    #[test]
    fn invalid_safe() {
        assert!(MediaFps::new(0, 1).is_none());
        assert!(MediaFps::new(1, 0).is_none());
    }

    #[test]
    fn interval() {
        let fps = MediaFps::new(30, 1).unwrap();
        assert!((fps.to_frame_interval() - (1.0 / 30.0)).abs() < 1e-9);
    }

    #[test]
    fn differential() {
        let fps = MediaFps::new(60, 1).unwrap();
        let got = fps.to_fps();
        assert_equal(got, 60.0, "fps conversion");
    }
}
