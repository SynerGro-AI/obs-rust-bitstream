//! Safe pure StatusMatch (ServiceReport ↔ FullStatus).
//!
//! True if the report name and attempts match the full status.
//! No unwraps.
//!
//! Differential: a different name fails the match.

use crate::full_status::FullStatus;
use crate::service_report::ServiceReport;
use crate::status_service::StatusService;

pub struct StatusMatch;

impl StatusMatch {
    /// True if the report matches the status.
    pub fn matches(status: &FullStatus, report: &ServiceReport) -> bool {
        let derived = StatusService::from(status);
        derived.name == report.name && derived.attempts == report.attempts
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
    fn matches_same() {
        let s = status("twitch", 1);
        let r = StatusService::from(&s);
        assert!(StatusMatch::matches(&s, &r));
    }

    #[test]
    fn differs() {
        let s = status("twitch", 1);
        let r = ServiceReport {
            name: "youtube".to_string(),
            attempts: 1,
        };
        assert!(!StatusMatch::matches(&s, &r));
    }

    #[test]
    fn differential() {
        let a = status("twitch", 1);
        let b = status("youtube", 1);
        let ra = StatusService::from(&a);
        let rb = StatusService::from(&b);
        assert!(StatusMatch::matches(&a, &ra));
        assert!(!StatusMatch::matches(&a, &rb));
    }
}
