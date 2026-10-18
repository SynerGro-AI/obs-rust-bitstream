//! Safe pure FullStatus (path + keyframe + caption + roi + reconnect + pause).
//!
//! One line of held, dropped, latency, keyframe done, caption lines,
//! ROI count, reconnect attempts, and pause state. No unwraps.
//!
//! Differential: a paused output reports paused=true.

use crate::caption_status::CaptionStatus;
use crate::keyframe_status::KeyframeStatus;
use crate::output_pause::OutputPause;
use crate::path_status::PathStatus;
use crate::reconnect::Reconnect;
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
    pub attempts: u32,
    pub paused: bool,
}

impl FullStatus {
    /// Build from the path, keyframe, caption, ROI, reconnect, and pause.
    pub fn from(
        path: &PathStatus,
        keyframe: &KeyframeStatus,
        caption: &CaptionStatus,
        roi: &RoiStatus,
        reconnect: &Reconnect,
        pause: &OutputPause,
    ) -> Self {
        Self {
            held: path.held,
            dropped: path.dropped,
            latency_ns: path.latency_ns,
            keyframe_done: keyframe.done,
            keyframe_complete: keyframe.complete,
            caption_lines: caption.lines,
            roi_count: roi.count,
            attempts: reconnect.attempts(),
            paused: pause.is_paused(),
        }
    }

    /// One line: "held=N dropped=N latency=N kf_done=N kf_complete=true/false captions=N roi=N attempts=N paused=true/false"
    pub fn one_line(&self) -> String {
        format!(
            "held={} dropped={} latency={} kf_done={} kf_complete={} captions={} roi={} attempts={} paused={}",
            self.held,
            self.dropped,
            self.latency_ns,
            self.keyframe_done,
            self.keyframe_complete,
            self.caption_lines,
            self.roi_count,
            self.attempts,
            self.paused
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
        let mut rc = Reconnect::new();
        rc.start();
        let mut pause = OutputPause::new();
        let out = crate::output_info::OutputInfo::new(
            "rec",
            "Recording",
            crate::output_info::OUTPUT_AV
                | crate::output_info::OUTPUT_ENCODED
                | crate::output_pause::OUTPUT_CAN_PAUSE,
        )
        .unwrap();
        pause.start(&out, 0).unwrap();
        let s = FullStatus::from(&p, &k, &c, &r, &rc, &pause);
        assert_eq!(s.held, 1);
        assert_eq!(s.attempts, 1);
        assert!(s.paused);
        assert!(s.one_line().contains("paused=true"));
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
        let rc = Reconnect::new();
        let pause = OutputPause::new();
        let s = FullStatus::from(&p, &k, &c, &r, &rc, &pause);
        assert_eq!(s.held, 0);
        assert!(!s.paused);
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
        let r = RoiStatus {
            count: 0,
            supported: false,
        };
        let rc = Reconnect::new();
        let unpaused = OutputPause::new();
        let mut paused = OutputPause::new();
        let out = crate::output_info::OutputInfo::new(
            "rec",
            "Recording",
            crate::output_info::OUTPUT_AV
                | crate::output_info::OUTPUT_ENCODED
                | crate::output_pause::OUTPUT_CAN_PAUSE,
        )
        .unwrap();
        paused.start(&out, 0).unwrap();
        let sa = FullStatus::from(&p, &k, &c, &r, &rc, &unpaused);
        let sb = FullStatus::from(&p, &k, &c, &r, &rc, &paused);
        assert!(!sa.paused);
        assert!(sb.paused);
    }
}
