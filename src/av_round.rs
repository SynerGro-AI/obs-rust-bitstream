//! Safe pure AvRound (status + AV match).
//!
//! Writes the full status and the AV match flag.
//! Returns true if both read back correctly. No unwraps.
//!
//! Differential: a mismatch reads false.

use crate::av_file::AvFile;
use crate::av_status::AvStatus;
use crate::service_status::ServiceStatus;
use crate::full_status::FullStatus;
use crate::status_file::StatusFile;
use crate::status_read::StatusRead;

pub struct AvRound;

impl AvRound {
    /// True if the status held count and AV match both read back.
    pub fn matches(status: &FullStatus, av: &AvStatus) -> bool {
        let status_buf = StatusFile::write(status);
        let av_buf = AvFile::write(av);
        let held_ok = StatusRead::held(&status_buf) == Some(status.held);
        let av_ok = AvFile::read(&av_buf) == Some(av.matches);
        held_ok && av_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caption_status::CaptionStatus;
    use crate::keyframe_status::KeyframeStatus;
    use crate::output_pause::OutputPause;
    use crate::path_status::PathStatus;
    use crate::reconnect::Reconnect;
    use crate::service_status::ServiceStatus;

    fn status(held: usize) -> FullStatus {
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
        let s = status(2);
        let av = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: true,
        };
        assert!(AvRound::matches(&s, &av));
    }

    #[test]
    fn mismatch_reads_false() {
        let s = status(0);
        let av = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: false,
        };
        assert!(AvRound::matches(&s, &av));
        assert_eq!(AvFile::read(&AvFile::write(&av)), Some(false));
    }

    #[test]
    fn differential() {
        let a = status(1);
        let b = status(3);
        let av = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: true,
        };
        assert!(AvRound::matches(&a, &av));
        assert!(AvRound::matches(&b, &av));
        assert_ne!(a.held, b.held);
    }
}
