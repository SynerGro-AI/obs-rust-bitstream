//! Safe pure StatusHistory (last N status lines).
//!
//! Keeps the last N one-line summaries. No unwraps.
//!
//! Differential: a pushed line can be read back.

#[derive(Clone, Debug)]
pub struct StatusHistory {
    lines: Vec<String>,
    max: usize,
}

impl StatusHistory {
    pub fn new(max: usize) -> Self {
        Self {
            lines: Vec::new(),
            max: max.max(1),
        }
    }

    pub fn push(&mut self, line: &str) {
        if self.lines.len() >= self.max {
            self.lines.remove(0);
        }
        self.lines.push(line.to_string());
    }

    pub fn last(&self) -> Option<&str> {
        self.lines.last().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Write all lines to a writer.
    pub fn write<W: std::io::Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        for line in &self.lines {
            writeln!(out, "{}", line)?;
        }
        Ok(())
    }

    /// Read lines from a reader. Caps at max.
    pub fn read<R: std::io::Read>(max: usize, mut input: R) -> Result<Self, std::io::Error> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        let mut h = Self::new(max);
        for line in buf.lines() {
            if !line.is_empty() {
                h.push(line);
            }
        }
        Ok(h)
    }

    /// True if the last line matches.
    pub fn matches_last(&self, other: &Self) -> bool {
        self.last() == other.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut h = StatusHistory::new(2);
        h.push("a");
        h.push("b");
        h.push("c");
        assert_eq!(h.len(), 2);
        assert_eq!(h.last(), Some("c"));
        let mut buf = Vec::new();
        h.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("c"));
        let loaded = StatusHistory::read(2, std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.last(), Some("c"));
        assert!(h.matches_last(&loaded));
    }
}
