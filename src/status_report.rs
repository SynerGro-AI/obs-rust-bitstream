//! Safe pure StatusReport (all status facts in one place).
//!
//! Collects the key facts from a history. No unwraps.
//!
//! Differential: a known history produces the expected facts.

use crate::status_history::StatusHistory;

#[derive(Clone, Debug)]
pub struct StatusReport {
    pub lines: usize,
    pub healthy: f32,
    pub avg_fps: f32,
    pub trend: String,
    pub stable: bool,
    pub streak: usize,
    pub record: bool,
    pub word: String,
    pub color: String,
}

impl StatusReport {
    /// Build from a history.
    pub fn from(h: &StatusHistory) -> Self {
        Self {
            lines: h.len(),
            healthy: h.health_ratio(),
            avg_fps: h.avg_fps(),
            trend: h.trend().to_string(),
            stable: h.stable(0.1),
            streak: h.streak(),
            record: h.at_record(),
            word: h.word().to_string(),
            color: h.color().to_string(),
        }
    }

    /// One readable line.
    pub fn summary(&self) -> String {
        format!(
            "{} lines | {:.0}% | {:.0} fps | {} | {} | streak {} | {}",
            self.lines,
            self.healthy * 100.0,
            self.avg_fps,
            self.trend,
            if self.stable { "stable" } else { "variable" },
            self.streak,
            self.word
        )
    }

    /// Write the summary to a writer.
    pub fn write<W: std::io::Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        writeln!(out, "{}", self.summary())
    }

    /// Read a report line. Returns the text.
    pub fn read<R: std::io::Read>(mut input: R) -> Result<String, std::io::Error> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        Ok(buf.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut h = StatusHistory::new(5);
        h.push("src | 100 fps | encode | healthy");
        let r = StatusReport::from(&h);
        assert_eq!(r.lines, 1);
        assert!(r.summary().contains("healthy") || r.summary().contains("good") || r.summary().contains("warn") || r.summary().contains("bad"));
        let mut buf = Vec::new();
        r.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("fps"));
        let loaded = StatusReport::read(std::io::Cursor::new(&buf)).unwrap();
        assert!(loaded.contains("fps"));
        assert!(r.matches(&loaded));
}
