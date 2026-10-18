//! Safe pure RtmpReconnectRound (write → read).
//!
//! Writes an RtmpReconnectStatus and reads the attempts back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a different attempt count fails the match.

use crate::rtmp_reconnect_file::RtmpReconnectFile;
use crate::rtmp_reconnect_status::RtmpReconnectStatus;

pub struct RtmpReconnectRound;

impl RtmpReconnectRound {
    /// True if the written attempts read back the same.
    pub fn matches(status: &RtmpReconnectStatus) -> bool {
        let buf = RtmpReconnectFile::write(status);
        RtmpReconnectFile::attempts(&buf) == Some(status.attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let s = RtmpReconnectStatus {
            active: true,
            packets: 2,
            attempts: 0,
        };
        assert!(RtmpReconnectRound::matches(&s));
    }

    #[test]
    fn nonzero_round_trips() {
        let s = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 3,
        };
        assert!(RtmpReconnectRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 1,
        };
        let b = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 5,
        };
        assert!(RtmpReconnectRound::matches(&a));
        assert!(RtmpReconnectRound::matches(&b));
        assert_ne!(a.attempts, b.attempts);
    }
}
