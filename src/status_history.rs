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

    /// Remove all lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Count lines that contain "healthy".
    pub fn healthy_count(&self) -> usize {
        self.lines.iter().filter(|l| l.contains("healthy")).count()
    }

    /// Fraction of healthy lines. 0.0 if empty.
    pub fn health_ratio(&self) -> f32 {
        if self.lines.is_empty() {
            return 0.0;
        }
        self.healthy_count() as f32 / self.lines.len() as f32
    }

    /// Average fps from lines that contain "fps".
    pub fn avg_fps(&self) -> f32 {
        let mut sum = 0.0;
        let mut n = 0;
        for line in &self.lines {
            for part in line.split('|') {
                let part = part.trim();
                if let Some(num) = part.strip_suffix(" fps") {
                    if let Ok(v) = num.parse::<f32>() {
                        sum += v;
                        n += 1;
                    }
                }
            }
        }
        if n == 0 { 0.0 } else { sum / n as f32 }
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

    /// Append this history to an existing file.
    pub fn append_to(&self, path: &str) -> Result<(), std::io::Error> {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut f = OpenOptions::new().create(true).append(true).open(path)?;
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
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
        let path = "/tmp/hist_append.txt";
        let _ = std::fs::remove_file(path);
        h.append_to(path).unwrap();
        h.append_to(path).unwrap();
        let again = StatusHistory::read(10, std::fs::File::open(path).unwrap()).unwrap();
        assert!(again.len() >= 2);
        let mut h2 = h.clone();
        h2.clear();
        assert!(h2.is_empty());
        let mut h3 = StatusHistory::new(5);
        h3.push("a | healthy");
        h3.push("b | slow");
        assert_eq!(h3.healthy_count(), 1);
        assert!((h3.health_ratio() - 0.5).abs() < 0.01);
        let mut h4 = StatusHistory::new(5);
        h4.push("src | 100 fps | encode | healthy");
        h4.push("src | 200 fps | encode | healthy");
        assert!((h4.avg_fps() - 150.0).abs() < 0.1);
    }
}
