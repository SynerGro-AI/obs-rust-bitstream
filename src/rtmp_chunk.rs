//! Safe pure RtmpChunk (OBS librtmp packet → chunk bytes).
//!
//! Wraps a payload in a minimal RTMP chunk header.
//! Format 0: 1-byte basic + 11-byte message header.
//! Type 9 is video, type 8 is audio (OBS packet types).
//! No unwraps.
//!
//! Differential: a larger payload produces more chunk bytes.

/// Video message type (OBS / RTMP).
pub const RTMP_TYPE_VIDEO: u8 = 9;
/// Audio message type (OBS / RTMP).
pub const RTMP_TYPE_AUDIO: u8 = 8;

pub struct RtmpChunk;

impl RtmpChunk {
    /// Wrap `payload` in a format-0 chunk on channel `csid`.
    /// Returns the chunk bytes. Empty payload is still a valid header.
    pub fn wrap(csid: u8, msg_type: u8, timestamp: u32, payload: &[u8]) -> Vec<u8> {
        let cs = csid & 0x3f;
        let mut out = Vec::with_capacity(12 + payload.len());
        // Basic header: fmt=0, csid in low 6 bits.
        out.push(cs);
        // Timestamp (3 bytes, big-endian).
        out.push(((timestamp >> 16) & 0xff) as u8);
        out.push(((timestamp >> 8) & 0xff) as u8);
        out.push((timestamp & 0xff) as u8);
        // Length (3 bytes, big-endian).
        let len = payload.len() as u32;
        out.push(((len >> 16) & 0xff) as u8);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
        // Type id.
        out.push(msg_type);
        // Message stream id (little-endian, 0).
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(payload);
        out
    }

    /// Length of the header alone.
    pub fn header_len() -> usize {
        12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps() {
        let c = RtmpChunk::wrap(4, RTMP_TYPE_VIDEO, 1000, &[1, 2, 3]);
        assert_eq!(c.len(), 12 + 3);
        assert_eq!(c[0] & 0x3f, 4);
        assert_eq!(c[7], RTMP_TYPE_VIDEO);
    }

    #[test]
    fn empty_is_header() {
        let c = RtmpChunk::wrap(2, RTMP_TYPE_AUDIO, 0, &[]);
        assert_eq!(c.len(), 12);
        assert_eq!(c[7], RTMP_TYPE_AUDIO);
    }

    #[test]
    fn differential() {
        let a = RtmpChunk::wrap(4, RTMP_TYPE_VIDEO, 0, &[1]);
        let b = RtmpChunk::wrap(4, RTMP_TYPE_VIDEO, 0, &[1, 2, 3, 4]);
        assert!(b.len() > a.len());
    }
}
