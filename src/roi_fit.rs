//! Safe pure RoiFit (region fits the frame).
//!
//! Accepts an EncoderRoi only if its area is within the
//! frame size. Oversized regions are rejected. No unwraps.
//!
//! Differential: a smaller region fits; a larger one does not.

use crate::encoder_roi::EncoderRoi;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoiFit {
    pub fits: bool,
    pub roi_area: u64,
    pub frame_area: u64,
}

impl RoiFit {
    /// True if the region fits inside the frame.
    pub fn check(roi: &EncoderRoi, frame_w: u32, frame_h: u32) -> Self {
        let frame_area = frame_w as u64 * frame_h as u64;
        let roi_area = roi.area();
        let fits = roi.right() <= frame_w && roi.bottom() <= frame_h && roi_area <= frame_area;
        Self {
            fits,
            roi_area,
            frame_area,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits() {
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap();
        let f = RoiFit::check(&roi, 100, 100);
        assert!(f.fits);
        assert_eq!(f.roi_area, 100);
    }

    #[test]
    fn oversized() {
        let roi = EncoderRoi::new(0, 200, 0, 200, 0.0).unwrap();
        let f = RoiFit::check(&roi, 100, 100);
        assert!(!f.fits);
    }

    #[test]
    fn differential() {
        let small = EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap();
        let large = EncoderRoi::new(0, 50, 0, 50, 0.0).unwrap();
        let a = RoiFit::check(&small, 100, 100);
        let b = RoiFit::check(&large, 100, 100);
        assert!(a.fits);
        assert!(b.roi_area > a.roi_area);
    }
}
