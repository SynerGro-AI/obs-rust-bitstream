//! Safe pure RtmpPublish (OBS librtmp publish).
//!
//! Builds a minimal AMF0 publish command.
//! Command name "publish", transaction id 0, null, stream name, "live".
//! No unwraps.
//!
//! Differential: a longer stream name produces a larger body.

pub struct RtmpPublish;

impl RtmpPublish {
    /// Build the AMF0 publish body for `stream`.
    /// Type is "live" (OBS default for live output).
    pub fn body(stream: &str) -> Vec<u8> {
        let mut out = Vec::new();
        // String "publish"
        out.push(0x02);
        let name = b"publish";
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
        out.extend_from_slice(name);
        // Number 0 (transaction id)
        out.push(0x00);
        out.extend_from_slice(&0.0f64.to_be_bytes());
        // Null
        out.push(0x05);
        // Stream name
        out.push(0x02);
        out.extend_from_slice(&(stream.len() as u16).to_be_bytes());
        out.extend_from_slice(stream.as_bytes());
        // Type "live"
        out.push(0x02);
        let kind = b"live";
        out.extend_from_slice(&(kind.len() as u16).to_be_bytes());
        out.extend_from_slice(kind);
        out
    }

    /// True if the body starts with the publish string.
    pub fn is_publish(buf: &[u8]) -> bool {
        buf.len() > 10 && buf[0] == 0x02 && &buf[3..10] == b"publish"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let b = RtmpPublish::body("demo");
        assert!(RtmpPublish::is_publish(&b));
        assert!(b.windows(4).any(|w| w == b"live"));
    }

    #[test]
    fn empty_still_publish() {
        let b = RtmpPublish::body("");
        assert!(RtmpPublish::is_publish(&b));
    }

    #[test]
    fn differential() {
        let a = RtmpPublish::body("a");
        let b = RtmpPublish::body("demo");
        assert!(b.len() > a.len());
    }
}
