//! Safe pure RtmpHandshake (OBS librtmp C0/C1).
//!
//! Builds the client handshake: 1-byte version + 1536-byte C1.
//! Version is 3 (OBS / RTMP). C1 starts with timestamp then zeros.
//! No unwraps.
//!
//! Differential: the handshake is always 1537 bytes.

/// RTMP version byte (OBS uses 3).
pub const RTMP_VERSION: u8 = 3;
/// C1 size in bytes.
pub const RTMP_C1_SIZE: usize = 1536;

pub struct RtmpHandshake;

impl RtmpHandshake {
    /// Build C0 + C1. Total 1537 bytes.
    pub fn client(timestamp: u32) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + RTMP_C1_SIZE);
        out.push(RTMP_VERSION);
        out.push(((timestamp >> 24) & 0xff) as u8);
        out.push(((timestamp >> 16) & 0xff) as u8);
        out.push(((timestamp >> 8) & 0xff) as u8);
        out.push((timestamp & 0xff) as u8);
        // Remaining C1 is zeros (simple handshake, no digest).
        out.resize(1 + RTMP_C1_SIZE, 0);
        out
    }

    /// True if the buffer looks like a C0/C1.
    pub fn is_client(buf: &[u8]) -> bool {
        buf.len() == 1 + RTMP_C1_SIZE && buf[0] == RTMP_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let h = RtmpHandshake::client(1000);
        assert_eq!(h.len(), 1537);
        assert_eq!(h[0], RTMP_VERSION);
        assert!(RtmpHandshake::is_client(&h));
    }

    #[test]
    fn rejects_short() {
        assert!(!RtmpHandshake::is_client(&[3]));
        assert!(!RtmpHandshake::is_client(&[]));
    }

    #[test]
    fn differential() {
        let a = RtmpHandshake::client(0);
        let b = RtmpHandshake::client(5000);
        assert_eq!(a.len(), b.len());
        assert_ne!(a[1..5], b[1..5]);
    }
}
