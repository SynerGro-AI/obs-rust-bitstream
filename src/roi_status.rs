//! Safe pure RoiStatus (region count report).
//!
//! One line of the ROI list length and whether the
//! encoder supports ROI. No unwraps.
//!
//! Differential: a caps with ROI reports more regions.

use crate::encoder_caps::{EncoderCaps, CAP_ROI};
use crate::roi_list::RoiList;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoiStatus {
    pub count: usize,
    pub supported: bool,
}

impl RoiStatus {
    /// Build from the current list and caps.
    pub fn from(list: &RoiList, caps: &EncoderCaps) -> Self {
        Self {
            count: list.len(),
            supported: caps.has(CAP_ROI),
        }
    }

    /// One line: "count=N supported=true/false"
    pub fn one_line(&self) -> String {
        format!("count={} supported={}", self.count, self.supported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_roi::EncoderRoi;

    #[test]
    fn reports() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        let mut list = RoiList::new();
        list.add(&c, EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap());
        let s = RoiStatus::from(&list, &c);
        assert_eq!(s.count, 1);
        assert!(s.supported);
        assert!(s.one_line().contains("count=1"));
    }

    #[test]
    fn unsupported_is_zero() {
        let c = EncoderCaps::new();
        let mut list = RoiList::new();
        list.add(&c, EncoderRoi::new(0, 10, 0, 10, 0.0).unwrap());
        let s = RoiStatus::from(&list, &c);
        assert_eq!(s.count, 0);
        assert!(!s.supported);
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
        let sa = RoiStatus::from(&a, &with);
        let sb = RoiStatus::from(&b, &without);
        assert!(sa.count > sb.count);
    }
}
