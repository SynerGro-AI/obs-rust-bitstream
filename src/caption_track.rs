//! Safe pure CaptionTrack (OBS caption_track_data).
//!
//! Parsed from libobs/obs-internal.h.
//! Holds caption lines with display duration.
//! Lines are ordered. Empty text is rejected. No unwraps.
//!
//! Differential: a longer line produces a larger payload.

/// Matching OBS CAPTION_LINE_CHARS.
pub const CAPTION_LINE_CHARS: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptionError {
    Empty,
    TooLong,
}

#[derive(Clone, Debug)]
pub struct CaptionLine {
    text: String,
    duration: f64,
}

impl CaptionLine {
    /// Create a line. Empty or too-long text is rejected.
    pub fn new(text: &str, duration: f64) -> Result<Self, CaptionError> {
        if text.is_empty() {
            return Err(CaptionError::Empty);
        }
        if text.chars().count() > CAPTION_LINE_CHARS {
            return Err(CaptionError::TooLong);
        }
        Ok(Self {
            text: text.to_string(),
            duration: duration.max(0.0),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }
}

#[derive(Clone, Debug, Default)]
pub struct CaptionTrack {
    lines: Vec<CaptionLine>,
    timestamp: f64,
}

impl CaptionTrack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, ts: f64) {
        self.timestamp = ts.max(0.0);
    }

    /// Append a line. Returns error if the line is invalid.
    pub fn push(&mut self, text: &str, duration: f64) -> Result<(), CaptionError> {
        let line = CaptionLine::new(text, duration)?;
        self.lines.push(line);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Total display duration of all lines.
    pub fn total_duration(&self) -> f64 {
        self.lines.iter().map(|l| l.duration).sum()
    }

    pub fn line(&self, index: usize) -> Option<&CaptionLine> {
        self.lines.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_line() {
        let mut t = CaptionTrack::new();
        t.push("hello", 2.0).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t.line(0).unwrap().text(), "hello");
        assert_eq!(t.total_duration(), 2.0);
    }

    #[test]
    fn rejects_empty_and_long() {
        let mut t = CaptionTrack::new();
        assert!(t.push("", 1.0).is_err());
        let long = "a".repeat(CAPTION_LINE_CHARS + 1);
        assert!(t.push(&long, 1.0).is_err());
    }

    #[test]
    fn timestamp() {
        let mut t = CaptionTrack::new();
        t.set_timestamp(5.5);
        assert_eq!(t.timestamp(), 5.5);
    }

    #[test]
    fn differential() {
        let mut a = CaptionTrack::new();
        let mut b = CaptionTrack::new();
        a.push("a", 1.0).unwrap();
        b.push("bb", 1.0).unwrap();
        assert!(b.line(0).unwrap().text().len() > a.line(0).unwrap().text().len());
    }
}
