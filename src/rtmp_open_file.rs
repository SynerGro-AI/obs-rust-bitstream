//! Safe pure RtmpOpenFile (open status → bytes and back).
//!
//! Writes an RtmpOpenStatus and reads the byte total back.
//! Returns None if malformed. No unwraps.
//!
//! Differential: a larger total reads as a larger value.

use crate::rtmp_open_status::RtmpOpenStatus;

pub struct RtmpOpenFile;

impl RtmpOpenFile {
    /// Write the one-line report.
    pub fn write(status: &RtmpOpenStatus) -> Vec<u8> {
        status.one_line().into_bytes()
    }

    /// Read the byte total. Returns None if malformed.
    pub fn bytes(buf: &[u8]) -> Option<u64> {
        let text = std::str::from_utf8(buf).ok()?;
        let start = text.find("bytes=")?;
        let rest = &text[start + 6..];
        let end = rest.find(' ').unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    /// True if the written total reads back the same.
    pub fn matches(status: &RtmpOpenStatus) -> bool {
        let buf = Self::write(status);
        Self::bytes(&buf) == Some(status.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let s = RtmpOpenStatus::from(2856);
        assert_eq!(RtmpOpenFile::bytes(&RtmpOpenFile::write(&s)), Some(2856));
        assert!(RtmpOpenFile::matches(&s));
    }

    #[test]
    fn bad_is_none() {
        assert!(RtmpOpenFile::bytes(b"bad").is_none());
        assert!(RtmpOpenFile::bytes(&[]).is_none());
    }

    #[test]
    fn differential() {
        let a = RtmpOpenStatus::from(1000);
        let b = RtmpOpenStatus::from(2856);
        let aa = RtmpOpenFile::bytes(&RtmpOpenFile::write(&a)).unwrap();
        let bb = RtmpOpenFile::bytes(&RtmpOpenFile::write(&b)).unwrap();
        assert!(bb > aa);
    }
}
