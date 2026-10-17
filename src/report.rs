//! Safe pure Report (status + rate core).
//!
//! Combines a system status and a rate comparison
//! into one report. No unwraps.
//!
//! Differential: the report contains both.

use crate::rate_compare::RateCompare;
use crate::system_status::SystemStatus;
use std::io::Write;

pub struct Report;

impl Report {
    /// Write status and rate to one writer.
    pub fn write<W: Write>(
        status: &SystemStatus,
        small: u32,
        mid: u32,
        large: u32,
        mut out: W,
    ) -> Result<(), std::io::Error> {
        writeln!(out, "{}", status.summary())?;
        RateCompare::write_three(small, mid, large, &mut out)
            .map_err(|_| std::io::Error::other("rate"))?;
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        writeln!(out, "time={}", secs)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av_bundle::AvBundle;

    #[test]
    fn differential() {
        let b = AvBundle::new();
        let s = SystemStatus::from_bundle(&b);
        let mut buf = Vec::new();
        Report::write(&s, 8, 16, 32, &mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);
        assert!(text.contains("encoder="));
        assert!(text.contains("32x32"));
        assert!(text.contains("time="));
    }
}
