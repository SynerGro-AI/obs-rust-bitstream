//! Safe pure FullStatus (path + keyframe + caption).
//!
//! One line of held, dropped, latency, keyframe done, and caption lines.
//! No unwraps.
//!
//! Differential: more caption lines produce a higher count.

use crate::caption_status::CaptionStatus;
use crate::keyframe_status::KeyframeStatus;
use crate::path_status::PathStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullStatus {
    pub held: usize,
    pub dropped: usize,
    pub latency_ns: u64,
    pub keyframe_done: usize,
    pub keyframe_complete: bool,
    pub caption_lines: usize,
}

impl FullStatus {
    /// Build from the path, keyframe, and caption statuses.
    pub fn from(path: &PathStatus, keyframe: &KeyframeStatus, caption: &CaptionStatus) -> Self {
        Self {
            held: path.held,
            dropped: path.dropped,
            latency_ns: path.latency_ns,
            keyframe_done: keyframe.done,
            keyframe_complete: keyframe.complete,
            caption_lines: caption.lines,
        }
    }

    /// One line: "held=N dropped=N latency=N kf_done=N kf_complete=true/false captions=N"
    pub fn one_line(&self) -> String {
        format!(
            "held={} dropped={} latency={} kf_done={} kf_complete={} captions={}",
            self.held,
            self.dropped,
            self.latency_ns,
            self.keyframe_done,
            self.keyframe_complete,
            self.caption_lines
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
        let s = FullStatus::from(&p, &k, &c);
        assert_eq!(s.held, 1);
        assert_eq!(s.keyframe_done, 1);
        assert_eq!(s.caption_lines, 2);
        assert!(!s.keyframe_complete);
        assert!(s.one_line().contains("captions=2"));
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
        let s = FullStatus::from(&p, &k, &c);
        assert_eq!(s.held, 0);
        assert_eq!(s.caption_lines, 0);
        assert!(s.keyframe_complete);
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
        let a = CaptionStatus {
            lines: 1,
            duration: 1.0,
        };
        let b = CaptionStatus {
            lines: 3,
            duration: 3.0,
        };
        let sa = FullStatus::from(&p, &k, &a);
        let sb = FullStatus::from(&p, &k, &b);
        assert!(sb.caption_lines > sa.caption_lines);
    }
}
