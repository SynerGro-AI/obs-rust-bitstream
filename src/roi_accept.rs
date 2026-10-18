//! Safe pure RoiAccept (ROI only if caps allow).
//!
//! Accepts an EncoderRoi only if the encoder caps include
//! CAP_ROI. Otherwise returns None. No unwraps.
//!
//! Differential: a caps with ROI accepts; one without does not.

use crate::encoder_caps::{EncoderCaps, CAP_ROI};
use crate::encoder_roi::EncoderRoi;

pub struct RoiAccept;

impl RoiAccept {
    /// Return the ROI if the caps support it. Otherwise None.
    pub fn accept(caps: &EncoderCaps, roi: EncoderRoi) -> Option<EncoderRoi> {
        if caps.has(CAP_ROI) {
            Some(roi)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_with_cap() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(RoiAccept::accept(&c, roi).is_some());
    }

    #[test]
    fn rejects_without() {
        let c = EncoderCaps::new();
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(RoiAccept::accept(&c, roi).is_none());
    }

    #[test]
    fn differential() {
        let mut with = EncoderCaps::new();
        with.set(CAP_ROI);
        let without = EncoderCaps::new();
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap();
        let a = RoiAccept::accept(&with, roi.clone());
        let b = RoiAccept::accept(&without, roi);
        assert!(a.is_some());
        assert!(b.is_none());
    }
}
