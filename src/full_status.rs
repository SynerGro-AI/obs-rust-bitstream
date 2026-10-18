//! Safe pure FullStatus (path + keyframe + caption + roi).
//!
//! One line of held, dropped, latency, keyframe done, caption lines, and ROI count.
//! No unwraps.
//!
//! Differential: more ROIs produce a higher count.

use crate::caption_status::CaptionStatus;
use crate::keyframe_status::KeyframeStatus;
use crate::path_status::PathStatus;
use crate::roi_status::RoiStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullStatus {
    pub held: usize,
    pub dropped: usize,
    pub latency_ns: u64,
    pub keyframe_done: usize,
    pub keyframe_complete: bool,
    pub caption_lines: usize,
    pub roi_count: usize,
}

impl FullStatus {
    /// Build from the path, keyframe, caption, and ROI statuses.
    pub fn from(
        path: &PathStatus,
        keyframe: &KeyframeStatus,
        caption: &CaptionStatus,
        roi: &RoiStatus,
    ) -> Self {
        Self {
            held: path.held,
            dropped: path.dropped,
            latency_ns: path.latency_ns,
            keyframe_done: keyframe.done,
            keyframe_complete: keyframe.complete,
            caption_lines: caption.lines,
            roi_count: roi.count,
        }
    }

    /// One line: "held=N dropped=N latency=N kf_done=N kf_complete=true/false captions=N roi=N"
    pub fn one_line(&self) -> String {
        format!(
            "held={} dropped={} latency={} kf_done={} kf_complete={} captions={} roi={}",
            self.held,
            self.dropped,
            self.latency_ns,
            self.keyframe_done,
            self.keyframe_complete,
            self.caption_lines,
            self.roi_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let p = PathStatus {
            held: 1,
            dropped: 0,
            latency_ns: 200,
        };
        let k = KeyframeStatus {
            required: 2,
            done: 1,
            complete: false,
        };
        let c = CaptionStatus {
            lines: 2,
            duration: 3.0,
        };
        let r = RoiStatus {
            count: 1,
            supported: true,
        };
        let s = FullStatus::from(&p, &k, &c, &r);
        assert_eq!(s.held, 1);
        assert_eq!(s.caption_lines, 2);
        assert_eq!(s.roi_count, 1);
        assert!(s.one_line().contains("roi=1"));
    }

    #[test]
    fn zero_is_zero() {
        let p = PathStatus {
            held: 0,
            dropped: 0,
            latency_ns: 0,
        };
        let k = KeyframeStatus {
            required: 0,
            done: 0,
            complete: true,
        };
        let c = CaptionStatus {
            lines: 0,
            duration: 0.0,
        };
        let r = RoiStatus {
            count: 0,
            supported: false,
        };
        let s = FullStatus::from(&p, &k, &c, &r);
        assert_eq!(s.held, 0);
        assert_eq!(s.roi_count, 0);
    }

    #[test]
    fn differential() {
        let p = PathStatus {
            held: 0,
            dropped: 0,
            latency_ns: 0,
        };
        let k = KeyframeStatus {
            required: 0,
            done: 0,
            complete: true,
        };
        let c = CaptionStatus {
            lines: 0,
            duration: 0.0,
        };
        let a = RoiStatus {
            count: 1,
            supported: true,
        };
        let b = RoiStatus {
            count: 3,
            supported: true,
        };
        let sa = FullStatus::from(&p, &k, &c, &a);
        let sb = FullStatus::from(&p, &k, &c, &b);
        assert!(sb.roi_count > sa.roi_count);
    }
}
