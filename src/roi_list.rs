//! Safe pure RoiList (accepted regions only).
//!
//! Adds an EncoderRoi only if the encoder caps include
//! CAP_ROI. Rejected regions are not stored. No unwraps.
//!
//! Differential: a caps with ROI keeps more regions.

use crate::encoder_caps::EncoderCaps;
#[cfg(test)]
use crate::encoder_caps::CAP_ROI;
use crate::encoder_roi::EncoderRoi;
use crate::roi_accept::RoiAccept;

#[derive(Clone, Debug, Default)]
pub struct RoiList {
    regions: Vec<EncoderRoi>,
}

impl RoiList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a region if the caps support ROI.
    /// Returns true if the region was stored.
    pub fn add(&mut self, caps: &EncoderCaps, roi: EncoderRoi) -> bool {
        if let Some(r) = RoiAccept::accept(caps, roi) {
            self.regions.push(r);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&EncoderRoi> {
        self.regions.get(index)
    }

    /// Clear all regions.
    pub fn clear(&mut self) {
        self.regions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_with_cap() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let mut list = RoiList::new();
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(list.add(&c, roi));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn rejects_without() {
        let c = EncoderCaps::new();
        let mut list = RoiList::new();
        let roi = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(!list.add(&c, roi));
        assert!(list.is_empty());
    }

    #[test]
    fn differential() {
        let mut with = EncoderCaps::new();
        with.set(CAP_ROI);
        let without = EncoderCaps::new();
        let mut a = RoiList::new();
        let mut b = RoiList::new();
        a.add(&with, EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap());
        a.add(&with, EncoderRoi::new(10, 20, 0, 10, 0.0).unwrap());
        b.add(&without, EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap());
        assert!(a.len() > b.len());
    }
}
