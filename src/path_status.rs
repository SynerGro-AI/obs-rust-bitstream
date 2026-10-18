//! Safe pure PathStatus (delay + drop + latency).
//!
//! One line of held packets, dropped count, and latency.
//! No unwraps.
//!
//! Differential: more held packets produce a larger held count.

use crate::delay_status::DelayStatus;
use crate::drop_status::DropStatus;
use crate::latency_status::LatencyStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathStatus {
    pub held: usize,
    pub dropped: usize,
    pub latency_ns: u64,
}

impl PathStatus {
    /// Build from the three status pieces.
    pub fn from(delay: &DelayStatus, drop: &DropStatus, latency: &LatencyStatus) -> Self {
        Self {
            held: delay.held,
            dropped: drop.dropped,
            latency_ns: latency.latency_ns,
        }
    }

    /// One line: "held=N dropped=N latency=N"
    pub fn one_line(&self) -> String {
        format!(
            "held={} dropped={} latency={}",
            self.held, self.dropped, self.latency_ns
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let d = DelayStatus {
            held: 2,
            paused: false,
            attempts: 0,
        };
        let dr = DropStatus {
            dropped: 1,
            kept: 3,
        };
        let l = LatencyStatus {
            encode_ns: 100,
            latency_ns: 500,
        };
        let s = PathStatus::from(&d, &dr, &l);
        assert_eq!(s.held, 2);
        assert_eq!(s.dropped, 1);
        assert_eq!(s.latency_ns, 500);
        assert!(s.one_line().contains("held=2"));
    }

    #[test]
    fn zero_is_zero() {
        let d = DelayStatus {
            held: 0,
            paused: false,
            attempts: 0,
        };
        let dr = DropStatus {
            dropped: 0,
            kept: 0,
        };
        let l = LatencyStatus {
            encode_ns: 0,
            latency_ns: 0,
        };
        let s = PathStatus::from(&d, &dr, &l);
        assert_eq!(s.held, 0);
        assert_eq!(s.dropped, 0);
        assert_eq!(s.latency_ns, 0);
    }

    #[test]
    fn differential() {
        let a = DelayStatus {
            held: 1,
            paused: false,
            attempts: 0,
        };
        let b = DelayStatus {
            held: 4,
            paused: false,
            attempts: 0,
        };
        let dr = DropStatus {
            dropped: 0,
            kept: 0,
        };
        let l = LatencyStatus {
            encode_ns: 0,
            latency_ns: 0,
        };
        let sa = PathStatus::from(&a, &dr, &l);
        let sb = PathStatus::from(&b, &dr, &l);
        assert!(sb.held > sa.held);
    }
}
