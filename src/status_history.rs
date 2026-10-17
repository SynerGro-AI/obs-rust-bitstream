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

    /// Oldest line.
    pub fn first(&self) -> Option<&str> {
        self.lines.first().map(|s| s.as_str())
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

    /// Min and max fps. (0.0, 0.0) if none.
    pub fn fps_range(&self) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = 0.0;
        let mut n = 0;
        for line in &self.lines {
            for part in line.split('|') {
                let part = part.trim();
                if let Some(num) = part.strip_suffix(" fps") {
                    if let Ok(v) = num.parse::<f32>() {
                        if v < min { min = v; }
                        if v > max { max = v; }
                        n += 1;
                    }
                }
            }
        }
        if n == 0 { (0.0, 0.0) } else { (min, max) }
    }

    /// Difference between max and min fps.
    pub fn fps_spread(&self) -> f32 {
        let (lo, hi) = self.fps_range();
        hi - lo
    }

    /// True if the spread is within a fraction of the average.
    pub fn stable(&self, fraction: f32) -> bool {
        let avg = self.avg_fps();
        if avg == 0.0 {
            return true;
        }
        self.fps_spread() <= avg * fraction
    }

    /// "up" if latest is above average, "down" if below, "flat" otherwise.
    pub fn trend(&self) -> &'static str {
        let avg = self.avg_fps();
        if avg == 0.0 {
            return "flat";
        }
        // Parse the last line's fps.
        let last_fps = self.last().and_then(|line| {
            line.split('|').find_map(|part| {
                let part = part.trim();
                part.strip_suffix(" fps").and_then(|n| n.parse::<f32>().ok())
            })
        });
        match last_fps {
            Some(v) if v > avg * 1.05 => "up",
            Some(v) if v < avg * 0.95 => "down",
            _ => "flat",
        }
    }

    /// One readable line for the history.
    pub fn summary(&self) -> String {
        format!(
            "{} lines | {:.0}% healthy | {:.0} fps avg | {}",
            self.len(),
            self.health_ratio() * 100.0,
            self.avg_fps(),
            self.trend()
        )
    }

    /// Write all lines to a writer.
    pub fn write<W: std::io::Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        writeln!(out, "# {}", self.summary())?;
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
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            h.push(line);
        }
        Ok(h)
    }

    /// Read only the header line (starts with #).
    pub fn read_header<R: std::io::Read>(mut input: R) -> Result<String, std::io::Error> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        for line in buf.lines() {
            if let Some(rest) = line.strip_prefix("# ") {
                return Ok(rest.to_string());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no header",
        ))
    }

    /// True if the last line matches.
    pub fn matches_last(&self, other: &Self) -> bool {
        self.last() == other.last()
    }

    /// True if the header matches this summary.
    pub fn matches_header(&self, header: &str) -> bool {
        self.summary() == header
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
        assert_eq!(h.first(), Some("b"));
        let mut buf = Vec::new();
        h.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("c"));
        assert!(String::from_utf8_lossy(&buf).starts_with("# "));
        let loaded = StatusHistory::read(2, std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.last(), Some("c"));
        assert!(h.matches_last(&loaded));
        // header is skipped
        let with_header = "# summary\nc\n";
        let loaded2 = StatusHistory::read(2, std::io::Cursor::new(with_header)).unwrap();
        assert_eq!(loaded2.last(), Some("c"));
        let header = StatusHistory::read_header(std::io::Cursor::new(with_header)).unwrap();
        assert!(header.contains("summary"));
        let mut h5 = StatusHistory::new(2);
        h5.push("c");
        assert!(!h5.matches_header("summary"));
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
        let (lo, hi) = h4.fps_range();
        assert!((lo - 100.0).abs() < 0.1);
        assert!((hi - 200.0).abs() < 0.1);
        assert!((h4.fps_spread() - 100.0).abs() < 0.1);
        assert!(!h4.stable(0.1)); // 100 spread on 150 avg is not stable
        assert!(h4.stable(1.0));
        assert_eq!(h4.trend(), "up");
        assert!(h4.summary().contains("up"));
    }
}
