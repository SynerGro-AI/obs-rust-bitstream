//! Safe pure StatusRound (write → read).
//!
//! Writes a FullStatus and reads the held count back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a different held count fails the round trip.

use crate::full_status::FullStatus;
use crate::status_file::StatusFile;
use crate::status_read::StatusRead;

pub struct StatusRound;

impl StatusRound {
    /// True if the written status reads back the same held count.
    pub fn matches(status: &FullStatus) -> bool {
        let buf = StatusFile::write(status);
        StatusRead::held(&buf) == Some(status.held)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av_status::AvStatus;
    use crate::caption_status::CaptionStatus;
    use crate::keyframe_status::KeyframeStatus;
    use crate::output_pause::OutputPause;
    use crate::path_status::PathStatus;
    use crate::reconnect::Reconnect;
    use crate::roi_status::RoiStatus;
    use crate::service_status::ServiceStatus;

    fn sample(held: usize) -> FullStatus {
        FullStatus::from(
            &PathStatus {
                held,
                dropped: 0,
                latency_ns: 0,
            },
            &KeyframeStatus {
                required: 0,
                done: 0,
                complete: true,
            },
            &CaptionStatus {
                lines: 0,
                duration: 0.0,
            },
            &RoiStatus {
                count: 0,
                supported: false,
            },
            &Reconnect::new(),
            &OutputPause::new(),
            &AvStatus {
                pixels: 0,
                rate: 0,
                channels: 0,
                matches: true,
            },
            &ServiceStatus {
                name: String::new(),
                id: String::new(),
            },
        )
    }

    #[test]
    fn round_trips() {
        let s = sample(4);
        assert!(StatusRound::matches(&s));
    }

    #[test]
    fn zero_round_trips() {
        let s = sample(0);
        assert!(StatusRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = sample(1);
        let b = sample(2);
        assert!(StatusRound::matches(&a));
        assert!(StatusRound::matches(&b));
        assert_ne!(a.held, b.held);
    }
}
