//! Safe pure RoiGate (caps and fit must both pass).
//!
//! Adds a region only if the encoder supports ROI and
//! the region fits the frame. Otherwise it is rejected.
//! No unwraps.
//!
//! Differential: a fitting region is stored; an oversized one is not.

use crate::encoder_caps::EncoderCaps;
use crate::encoder_roi::EncoderRoi;
use crate::roi_fit::RoiFit;
use crate::roi_list::RoiList;

pub struct RoiGate;

impl RoiGate {
    /// Add the region if caps allow and it fits the frame.
    /// Returns true if the region was stored.
    pub fn add(
        list: &mut RoiList,
        caps: &EncoderCaps,
        roi: EncoderRoi,
        frame_w: u32,
        frame_h: u32,
    ) -> bool {
        if !RoiFit::check(&roi, frame_w, frame_h).fits {
            return false;
        }
        list.add(caps, roi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_caps::CAP_ROI;

    #[test]
    fn fitting_stored() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let mut list = RoiList::new();
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(RoiGate::add(&mut list, &c, roi, 100, 100));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn oversized_rejected() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let mut list = RoiList::new();
        let roi = EncoderRoi::new(0, 200, 0, 200, 0.0).unwrap();
        assert!(!RoiGate::add(&mut list, &c, roi, 100, 100));
        assert!(list.is_empty());
    }

    #[test]
    fn differential() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let mut a = RoiList::new();
        let mut b = RoiList::new();
        RoiGate::add(
            &mut a,
            &c,
            EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap(),
            100,
            100,
        );
        RoiGate::add(
            &mut b,
            &c,
            EncoderRoi::new(0, 200, 0, 200, 0.0).unwrap(),
            100,
            100,
        );
        assert!(a.len() > b.len());
    }
}
