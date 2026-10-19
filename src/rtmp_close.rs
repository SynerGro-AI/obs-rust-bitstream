//! Safe pure RtmpClose (OBS librtmp deleteStream).
//!
//! Builds a minimal AMF0 deleteStream command.
//! Command name "deleteStream", transaction id 0, null, stream id.
//! No unwraps.
//!
//! Differential: a different stream id changes the number bytes.

pub struct RtmpClose;

impl RtmpClose {
    /// Build the AMF0 deleteStream body for `stream_id`.
    pub fn body(stream_id: f64) -> Vec<u8> {
        let mut out = Vec::new();
        // String "deleteStream"
        out.push(0x02);
        let name = b"deleteStream";
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
        out.extend_from_slice(name);
        // Number 0 (transaction id)
        out.push(0x00);
        out.extend_from_slice(&0.0f64.to_be_bytes());
        // Null
        out.push(0x05);
        // Stream id
        out.push(0x00);
        out.extend_from_slice(&stream_id.to_be_bytes());
        out
    }

    /// True if the body starts with the deleteStream string.
    pub fn is_close(buf: &[u8]) -> bool {
        buf.len() > 15 && buf[0] == 0x02 && &buf[3..15] == b"deleteStream"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let b = RtmpClose::body(1.0);
        assert!(RtmpClose::is_close(&b));
        assert!(b.len() > 20);
    }

    #[test]
    fn different_id() {
        let a = RtmpClose::body(1.0);
        let b = RtmpClose::body(2.0);
        assert_ne!(a, b);
        assert!(RtmpClose::is_close(&b));
    }

    #[test]
    fn differential() {
        let a = RtmpClose::body(0.0);
        let b = RtmpClose::body(1.0);
        assert_eq!(a.len(), b.len());
        assert_ne!(a, b);
    }
}
