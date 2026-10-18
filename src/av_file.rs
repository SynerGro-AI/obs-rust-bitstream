//! Safe pure AvFile (AV status → bytes and back).
//!
//! Writes an AvStatus match flag and reads it back.
//! Returns None if the buffer is malformed. No unwraps.
//!
//! Differential: a match reads true; a mismatch reads false.

use crate::av_status::AvStatus;

pub struct AvFile;

impl AvFile {
    /// Write the match flag as "av=1" or "av=0".
    pub fn write(status: &AvStatus) -> Vec<u8> {
        let v = if status.matches { 1 } else { 0 };
        format!("av={v}").into_bytes()
    }

    /// Read the match flag. Returns None if malformed.
    pub fn read(buf: &[u8]) -> Option<bool> {
        let text = std::str::from_utf8(buf).ok()?;
        if !text.starts_with("av=") {
            return None;
        }
        match &text[3..] {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_true() {
        let s = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: true,
        };
        let buf = AvFile::write(&s);
        assert_eq!(AvFile::read(&buf), Some(true));
    }

    #[test]
    fn write_read_false() {
        let s = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: false,
        };
        let buf = AvFile::write(&s);
        assert_eq!(AvFile::read(&buf), Some(false));
    }

    #[test]
    fn bad_is_none() {
        assert!(AvFile::read(b"bad").is_none());
        assert!(AvFile::read(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: false,
        };
        let b = AvStatus {
            pixels: 0,
            rate: 0,
            channels: 0,
            matches: true,
        };
        assert_eq!(AvFile::read(&AvFile::write(&a)), Some(false));
        assert_eq!(AvFile::read(&AvFile::write(&b)), Some(true));
    }
}
