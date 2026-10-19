//! Safe pure RtmpOpenRound (write → read).
//!
//! Writes an RtmpOpenStatus and reads the byte total back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a different total fails the match.

use crate::rtmp_open_file::RtmpOpenFile;
use crate::rtmp_open_status::RtmpOpenStatus;

pub struct RtmpOpenRound;

impl RtmpOpenRound {
    /// True if the written total reads back the same.
    pub fn matches(status: &RtmpOpenStatus) -> bool {
        RtmpOpenFile::matches(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let s = RtmpOpenStatus::from(2856);
        assert!(RtmpOpenRound::matches(&s));
    }

    #[test]
    fn zero_round_trips() {
        let s = RtmpOpenStatus::from(0);
        assert!(RtmpOpenRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = RtmpOpenStatus::from(1000);
        let b = RtmpOpenStatus::from(2856);
        assert!(RtmpOpenRound::matches(&a));
        assert!(RtmpOpenRound::matches(&b));
        assert_ne!(a.bytes, b.bytes);
    }
}
