//! Safe pure StreamFile (stream status → bytes and back).
//!
//! Writes a StreamStatus and reads the streaming flag back.
//! Returns None if malformed. No unwraps.
//!
//! Differential: a running stream reads true.

use crate::stream_status::StreamStatus;

pub struct StreamFile;

impl StreamFile {
    /// Write as "streaming=1/0 service=N bitrate=N".
    pub fn write(status: &StreamStatus) -> Vec<u8> {
        status.one_line().into_bytes()
    }

    /// Read the streaming flag. Returns None if malformed.
    pub fn streaming(buf: &[u8]) -> Option<bool> {
        let text = std::str::from_utf8(buf).ok()?;
        if !text.starts_with("streaming=") {
            return None;
        }
        match text.as_bytes().get(10) {
            Some(b'1') => Some(true),
            Some(b'0') => Some(false),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_true() {
        let s = StreamStatus {
            streaming: true,
            service: "twitch".to_string(),
            bitrate_kbps: 6000,
        };
        assert_eq!(StreamFile::streaming(&StreamFile::write(&s)), Some(true));
    }

    #[test]
    fn write_read_false() {
        let s = StreamStatus {
            streaming: false,
            service: String::new(),
            bitrate_kbps: 0,
        };
        assert_eq!(StreamFile::streaming(&StreamFile::write(&s)), Some(false));
    }

    #[test]
    fn bad_is_none() {
        assert!(StreamFile::streaming(b"bad").is_none());
        assert!(StreamFile::streaming(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = StreamStatus {
            streaming: false,
            service: String::new(),
            bitrate_kbps: 0,
        };
        let b = StreamStatus {
            streaming: true,
            service: "youtube".to_string(),
            bitrate_kbps: 6000,
        };
        assert_eq!(StreamFile::streaming(&StreamFile::write(&a)), Some(false));
        assert_eq!(StreamFile::streaming(&StreamFile::write(&b)), Some(true));
    }
}
