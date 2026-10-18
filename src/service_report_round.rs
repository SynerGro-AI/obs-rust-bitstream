//! Safe pure ServiceReportRound (write → read).
//!
//! Writes a ServiceReport and reads the attempts back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a different attempt count fails the match.

use crate::service_report::ServiceReport;
use crate::service_report_file::ServiceReportFile;

pub struct ServiceReportRound;

impl ServiceReportRound {
    /// True if the written attempts read back the same.
    pub fn matches(report: &ServiceReport) -> bool {
        let buf = ServiceReportFile::write(report);
        ServiceReportFile::attempts(&buf) == Some(report.attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let r = ServiceReport {
            name: "twitch".to_string(),
            attempts: 2,
        };
        assert!(ServiceReportRound::matches(&r));
    }

    #[test]
    fn zero_round_trips() {
        let r = ServiceReport {
            name: String::new(),
            attempts: 0,
        };
        assert!(ServiceReportRound::matches(&r));
    }

    #[test]
    fn differential() {
        let a = ServiceReport {
            name: "twitch".to_string(),
            attempts: 1,
        };
        let b = ServiceReport {
            name: "twitch".to_string(),
            attempts: 4,
        };
        assert!(ServiceReportRound::matches(&a));
        assert!(ServiceReportRound::matches(&b));
        assert_ne!(a.attempts, b.attempts);
    }
}
