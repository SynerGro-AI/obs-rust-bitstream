//! Safe pure ServiceReportFile (report → bytes and back).
//!
//! Writes a ServiceReport and reads the attempts back.
//! Returns None if malformed. No unwraps.
//!
//! Differential: more attempts read as a larger value.

use crate::service_report::ServiceReport;

pub struct ServiceReportFile;

impl ServiceReportFile {
    /// Write the report as "name=N attempts=N".
    pub fn write(report: &ServiceReport) -> Vec<u8> {
        report.one_line().into_bytes()
    }

    /// Read the attempts. Returns None if malformed.
    pub fn attempts(buf: &[u8]) -> Option<u32> {
        let text = std::str::from_utf8(buf).ok()?;
        let start = text.find("attempts=")?;
        let rest = &text[start + 9..];
        let end = rest.find(' ').unwrap_or(rest.len());
        rest[..end].parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let r = ServiceReport {
            name: "twitch".to_string(),
            attempts: 2,
        };
        let buf = ServiceReportFile::write(&r);
        assert_eq!(ServiceReportFile::attempts(&buf), Some(2));
    }

    #[test]
    fn zero_reads_zero() {
        let r = ServiceReport {
            name: String::new(),
            attempts: 0,
        };
        let buf = ServiceReportFile::write(&r);
        assert_eq!(ServiceReportFile::attempts(&buf), Some(0));
    }

    #[test]
    fn bad_is_none() {
        assert!(ServiceReportFile::attempts(b"bad").is_none());
        assert!(ServiceReportFile::attempts(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = ServiceReport {
            name: "twitch".to_string(),
            attempts: 1,
        };
        let b = ServiceReport {
            name: "twitch".to_string(),
            attempts: 3,
        };
        let aa = ServiceReportFile::attempts(&ServiceReportFile::write(&a)).unwrap();
        let bb = ServiceReportFile::attempts(&ServiceReportFile::write(&b)).unwrap();
        assert!(bb > aa);
    }
}
