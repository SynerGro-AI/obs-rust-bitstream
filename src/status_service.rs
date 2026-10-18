//! Safe pure StatusService (FullStatus → ServiceReport).
//!
//! Builds a ServiceReport from a FullStatus.
//! Name and attempts come from the status. No unwraps.
//!
//! Differential: more attempts produce a higher count.

use crate::full_status::FullStatus;
use crate::service_report::ServiceReport;

pub struct StatusService;

impl StatusService {
    /// Build a ServiceReport from the full status.
    pub fn from(status: &FullStatus) -> ServiceReport {
        ServiceReport {
            name: status.service_name.clone(),
            attempts: status.attempts,
        }
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

    fn status(name: &str, attempts: u32) -> FullStatus {
        let mut rc = Reconnect::new();
        for _ in 0..attempts {
            rc.start();
        }
        FullStatus::from(
            &PathStatus {
                held: 0,
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
            &rc,
            &OutputPause::new(),
            &AvStatus {
                pixels: 0,
                rate: 0,
                channels: 0,
                matches: true,
            },
            &ServiceStatus {
                name: name.to_string(),
                id: String::new(),
            },
        )
    }

    #[test]
    fn builds() {
        let s = status("twitch", 2);
        let r = StatusService::from(&s);
        assert_eq!(r.name, "twitch");
        assert_eq!(r.attempts, 2);
    }

    #[test]
    fn empty_is_zero() {
        let s = status("", 0);
        let r = StatusService::from(&s);
        assert_eq!(r.name, "");
        assert_eq!(r.attempts, 0);
    }

    #[test]
    fn differential() {
        let a = StatusService::from(&status("twitch", 1));
        let b = StatusService::from(&status("twitch", 3));
        assert!(b.attempts > a.attempts);
    }
}
