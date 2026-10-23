//! Safe pure RtmpReconnectFile (status → bytes and back).
//!
//! Writes an RtmpReconnectStatus and reads the attempts back.
//! Returns None if malformed. No unwraps.
//!
//! Differential: more attempts read as a larger value.

use crate::rtmp_reconnect_status::RtmpReconnectStatus;

pub struct RtmpReconnectFile;

impl RtmpReconnectFile {
    /// Write the one-line report.
    pub fn write(status: &RtmpReconnectStatus) -> Vec<u8> {
        status.one_line().into_bytes()
    }

    /// Read the attempts. Returns None if malformed.
    pub fn attempts(buf: &[u8]) -> Option<u32> {
        let text = std::str::from_utf8(buf).ok()?;
        let start = text.find("attempts=")?;
        let rest = &text[start + 9..];
        let end = rest.find(' ').unwrap_or(rest.len());
        rest[..end].parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let s = RtmpReconnectStatus {
            active: true,
            packets: 2,
            attempts: 0,
        };
        assert_eq!(
            RtmpReconnectFile::attempts(&RtmpReconnectFile::write(&s)),
            Some(0)
        );
    }

    #[test]
    fn reads_nonzero() {
        let s = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 3,
        };
        assert_eq!(
            RtmpReconnectFile::attempts(&RtmpReconnectFile::write(&s)),
            Some(3)
        );
    }

    #[test]
    fn bad_is_none() {
        assert!(RtmpReconnectFile::attempts(b"bad").is_none());
        assert!(RtmpReconnectFile::attempts(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 1,
        };
        let b = RtmpReconnectStatus {
            active: false,
            packets: 0,
            attempts: 4,
        };
        let aa = RtmpReconnectFile::attempts(&RtmpReconnectFile::write(&a)).unwrap();
        let bb = RtmpReconnectFile::attempts(&RtmpReconnectFile::write(&b)).unwrap();
        assert!(bb > aa);
    }
}
