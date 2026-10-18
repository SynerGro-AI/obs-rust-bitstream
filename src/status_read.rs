//! Safe pure StatusRead (bytes → held count).
//!
//! Parses a FullStatus one-line report and returns the held count.
//! Returns None if the line is missing or malformed. No unwraps.
//!
//! Differential: a higher held count parses to a larger value.

use crate::status_file::StatusFile;

pub struct StatusRead;

impl StatusRead {
    /// Parse the held count from a status buffer.
    /// Returns None if the buffer is not a status line.
    pub fn held(buf: &[u8]) -> Option<usize> {
        if !StatusFile::is_status(buf) {
            return None;
        }
        let text = std::str::from_utf8(buf).ok()?;
        let start = text.find("held=")?;
        let rest = &text[start + 5..];
        let end = rest.find(' ').unwrap_or(rest.len());
        rest[..end].parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caption_status::CaptionStatus;
    use crate::full_status::FullStatus;
    use crate::keyframe_status::KeyframeStatus;
    use crate::output_pause::OutputPause;
    use crate::path_status::PathStatus;
    use crate::reconnect::Reconnect;
    use crate::roi_status::RoiStatus;

    fn sample(held: usize) -> FullStatus {
        FullStatus::from(
            &PathStatus {
                held,
                dropped: 0,
                latency_ns: 0,
            },
            &KeyframeStatus {
                required: 0,
                done: 0,
                complete: true,
            },
            &CaptionStatus {
                lines: 0,
                duration: 0.0,
            },
            &RoiStatus {
                count: 0,
                supported: false,
            },
            &Reconnect::new(),
            &OutputPause::new(),
        )
    }

    #[test]
    fn reads() {
        let s = sample(3);
        let buf = StatusFile::write(&s);
        assert_eq!(StatusRead::held(&buf), Some(3));
    }

    #[test]
    fn bad_is_none() {
        assert!(StatusRead::held(b"bad").is_none());
        assert!(StatusRead::held(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = StatusRead::held(&StatusFile::write(&sample(1))).unwrap();
        let b = StatusRead::held(&StatusFile::write(&sample(5))).unwrap();
        assert!(b > a);
    }
}
