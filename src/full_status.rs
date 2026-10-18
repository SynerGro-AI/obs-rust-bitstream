//! Safe pure FullStatus (path + keyframe).
//!
//! One line of held, dropped, latency, and keyframe done.
//! No unwraps.
//!
//! Differential: a more complete group reports a higher done.

use crate::keyframe_status::KeyframeStatus;
use crate::path_status::PathStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullStatus {
    pub held: usize,
    pub dropped: usize,
    pub latency_ns: u64,
    pub keyframe_done: usize,
    pub keyframe_complete: bool,
}

impl FullStatus {
    /// Build from the path and keyframe statuses.
    pub fn from(path: &PathStatus, keyframe: &KeyframeStatus) -> Self {
        Self {
            held: path.held,
            dropped: path.dropped,
            latency_ns: path.latency_ns,
            keyframe_done: keyframe.done,
            keyframe_complete: keyframe.complete,
        }
    }

    /// One line: "held=N dropped=N latency=N kf_done=N kf_complete=true/false"
    pub fn one_line(&self) -> String {
        format!(
            "held={} dropped={} latency={} kf_done={} kf_complete={}",
            self.held, self.dropped, self.latency_ns, self.keyframe_done, self.keyframe_complete
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
        let s = FullStatus::from(&p, &k);
        assert_eq!(s.held, 1);
        assert_eq!(s.keyframe_done, 1);
        assert!(!s.keyframe_complete);
        assert!(s.one_line().contains("kf_done=1"));
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
        let s = FullStatus::from(&p, &k);
        assert_eq!(s.held, 0);
        assert!(s.keyframe_complete);
    }

    #[test]
    fn differential() {
        let p = PathStatus {
            held: 0,
            dropped: 0,
            latency_ns: 0,
        };
        let a = KeyframeStatus {
            required: 2,
            done: 1,
            complete: false,
        };
        let b = KeyframeStatus {
            required: 2,
            done: 2,
            complete: true,
        };
        let sa = FullStatus::from(&p, &a);
        let sb = FullStatus::from(&p, &b);
        assert!(sb.keyframe_done > sa.keyframe_done);
    }
}
