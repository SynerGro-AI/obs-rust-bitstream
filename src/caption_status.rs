//! Safe pure CaptionStatus (line count report).
//!
//! One line of caption line count and total duration.
//! No unwraps.
//!
//! Differential: more lines produce a higher count.

use crate::caption_track::CaptionTrack;

#[derive(Clone, Debug, PartialEq)]
pub struct CaptionStatus {
    pub lines: usize,
    pub duration: f64,
}

impl CaptionStatus {
    /// Build from a CaptionTrack.
    pub fn from(track: &CaptionTrack) -> Self {
        Self {
            lines: track.len(),
            duration: track.total_duration(),
        }
    }

    /// One line: "lines=N duration=N"
    pub fn one_line(&self) -> String {
        format!("lines={} duration={}", self.lines, self.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let mut t = CaptionTrack::new();
        t.push("hello", 2.0).unwrap();
        t.push("world", 1.5).unwrap();
        let s = CaptionStatus::from(&t);
        assert_eq!(s.lines, 2);
        assert_eq!(s.duration, 3.5);
        assert!(s.one_line().contains("lines=2"));
    }

    #[test]
    fn empty_is_zero() {
        let t = CaptionTrack::new();
        let s = CaptionStatus::from(&t);
        assert_eq!(s.lines, 0);
        assert_eq!(s.duration, 0.0);
    }

    #[test]
    fn differential() {
        let mut a = CaptionTrack::new();
        let mut b = CaptionTrack::new();
        a.push("a", 1.0).unwrap();
        b.push("a", 1.0).unwrap();
        b.push("b", 1.0).unwrap();
        let sa = CaptionStatus::from(&a);
        let sb = CaptionStatus::from(&b);
        assert!(sb.lines > sa.lines);
    }
}
