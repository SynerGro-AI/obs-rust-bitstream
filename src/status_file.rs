//! Safe pure StatusFile (full status → bytes).
//!
//! Writes a FullStatus one-line report to a byte buffer.
//! Empty status produces a zero line. No unwraps.
//!
//! Differential: a higher held count produces a longer line.

use crate::full_status::FullStatus;

pub struct StatusFile;

impl StatusFile {
    /// Write the status as bytes.
    pub fn write(status: &FullStatus) -> Vec<u8> {
        status.one_line().into_bytes()
    }

    /// True if the buffer starts with "held=".
    pub fn is_status(buf: &[u8]) -> bool {
        buf.starts_with(b"held=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caption_status::CaptionStatus;
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
    fn writes() {
        let s = sample(1);
        let buf = StatusFile::write(&s);
        assert!(StatusFile::is_status(&buf));
        assert!(String::from_utf8_lossy(&buf).contains("held=1"));
    }

    #[test]
    fn zero_is_zero() {
        let s = sample(0);
        let buf = StatusFile::write(&s);
        assert!(String::from_utf8_lossy(&buf).contains("held=0"));
    }

    #[test]
    fn differential() {
        let a = StatusFile::write(&sample(1));
        let b = StatusFile::write(&sample(10));
        assert!(b.len() >= a.len());
    }
}
