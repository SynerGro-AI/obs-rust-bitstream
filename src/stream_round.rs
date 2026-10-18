//! Safe pure StreamRound (write → read).
//!
//! Writes a StreamStatus and reads the streaming flag back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a running stream reads true; a stopped one reads false.

use crate::stream_file::StreamFile;
use crate::stream_status::StreamStatus;

pub struct StreamRound;

impl StreamRound {
    /// True if the written flag reads back the same.
    pub fn matches(status: &StreamStatus) -> bool {
        let buf = StreamFile::write(status);
        StreamFile::streaming(&buf) == Some(status.streaming)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_true() {
        let s = StreamStatus {
            streaming: true,
            service: "twitch".to_string(),
            bitrate_kbps: 6000,
        };
        assert!(StreamRound::matches(&s));
    }

    #[test]
    fn round_trips_false() {
        let s = StreamStatus {
            streaming: false,
            service: String::new(),
            bitrate_kbps: 0,
        };
        assert!(StreamRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = StreamStatus {
            streaming: false,
            service: String::new(),
            bitrate_kbps: 0,
        };
        let b = StreamStatus {
            streaming: true,
            service: "youtube".to_string(),
            bitrate_kbps: 6000,
        };
        assert!(StreamRound::matches(&a));
        assert!(StreamRound::matches(&b));
        assert_ne!(a.streaming, b.streaming);
    }
}
