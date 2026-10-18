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
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;
    use crate::rtmp_reconnect::RtmpReconnect;

    #[test]
    fn reports_success() {
        let packets = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let mut r = Reconnect::new();
        let st = RtmpReconnect::feed("live.twitch.tv", "abc", &packets, &mut r).unwrap();
        let rs = RtmpReconnectStatus::from(&st, &r);
        assert!(rs.active);
        assert_eq!(rs.attempts, 0);
        assert!(rs.one_line().contains("attempts=0"));
    }

    #[test]
    fn reports_failure() {
        let mut r = Reconnect::new();
        let _ = RtmpReconnect::feed("", "abc", &[], &mut r);
        let empty = RtmpStatus {
            active: false,
            packets: 0,
            bytes: 0,
            url: String::new(),
        };
        let rs = RtmpReconnectStatus::from(&empty, &r);
        assert!(!rs.active);
        assert_eq!(rs.attempts, 1);
    }

    #[test]
    fn differential() {
        let mut a = Reconnect::new();
        let mut b = Reconnect::new();
        let _ = RtmpReconnect::feed("", "k", &[], &mut a);
        let _ = RtmpReconnect::feed("", "k", &[], &mut b);
        let _ = RtmpReconnect::feed("", "k", &[], &mut b);
        let empty = RtmpStatus {
            active: false,
            packets: 0,
            bytes: 0,
            url: String::new(),
        };
        let sa = RtmpReconnectStatus::from(&empty, &a);
        let sb = RtmpReconnectStatus::from(&empty, &b);
        assert!(sb.attempts > sa.attempts);
    }
}
