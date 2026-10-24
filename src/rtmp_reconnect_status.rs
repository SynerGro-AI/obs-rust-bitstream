//! Safe pure RtmpReconnectStatus (RTMP + attempts).
//!
//! One line of active, packets, and reconnect attempts.
//! No unwraps.
//!
//! Differential: more failures produce more attempts.

use crate::reconnect::Reconnect;
use crate::rtmp_status::RtmpStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RtmpReconnectStatus {
    pub active: bool,
    pub packets: usize,
    pub attempts: u32,
}

impl RtmpReconnectStatus {
    /// Build from the RTMP status and reconnect.
    pub fn from(status: &RtmpStatus, reconnect: &Reconnect) -> Self {
        Self {
            active: status.active,
            packets: status.packets,
            attempts: reconnect.attempts(),
        }
    }

    /// One line: "active=true/false packets=N attempts=N"
    pub fn one_line(&self) -> String {
        format!(
            "active={} packets={} attempts={}",
            self.active, self.packets, self.attempts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(active: bool, packets: usize) -> RtmpStatus {
        RtmpStatus {
            active,
            packets,
            bytes: packets as u64,
            url: String::new(),
        }
    }

    #[test]
    fn reports_success() {
        let mut r = Reconnect::new();
        r.start();
        r.success();
        let rs = RtmpReconnectStatus::from(&status(true, 1), &r);
        assert!(rs.active);
        assert_eq!(rs.attempts, 0);
        assert!(rs.one_line().contains("attempts=0"));
    }

    #[test]
    fn reports_failure() {
        let mut r = Reconnect::new();
        r.start();
        let rs = RtmpReconnectStatus::from(&status(false, 0), &r);
        assert!(!rs.active);
        assert_eq!(rs.attempts, 1);
    }

    #[test]
    fn differential() {
        let mut a = Reconnect::new();
        let mut b = Reconnect::new();
        a.start();
        b.start();
        b.start();
        let sa = RtmpReconnectStatus::from(&status(false, 0), &a);
        let sb = RtmpReconnectStatus::from(&status(false, 0), &b);
        assert!(sb.attempts > sa.attempts);
    }
}
