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
    }
}
