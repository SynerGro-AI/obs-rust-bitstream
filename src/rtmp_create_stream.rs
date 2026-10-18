//! Safe pure RtmpCreateStream (OBS librtmp createStream).
//!
//! Builds a minimal AMF0 createStream command.
//! Command name "createStream", transaction id 2, null object.
//! No unwraps.
//!
//! Differential: the body is fixed size; a different
//! transaction id changes the number bytes.

pub struct RtmpCreateStream;

impl RtmpCreateStream {
    /// Build the AMF0 createStream body.
    /// Transaction id defaults to 2 (OBS uses 2 after connect=1).
    pub fn body(transaction_id: f64) -> Vec<u8> {
        let mut out = Vec::new();
        // String "createStream"
        out.push(0x02);
        let name = b"createStream";
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
        out.extend_from_slice(name);
        // Number (transaction id)
        out.push(0x00);
        out.extend_from_slice(&transaction_id.to_be_bytes());
        // Null
        out.push(0x05);
        out
    }

    /// True if the body starts with the createStream string.
    pub fn is_create_stream(buf: &[u8]) -> bool {
        buf.len() > 15 && buf[0] == 0x02 && &buf[3..15] == b"createStream"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let b = RtmpCreateStream::body(2.0);
        assert!(RtmpCreateStream::is_create_stream(&b));
        assert!(b.len() > 16);
    }

    #[test]
    fn different_id() {
        let a = RtmpCreateStream::body(2.0);
        let b = RtmpCreateStream::body(3.0);
        assert_ne!(a, b);
        assert!(RtmpCreateStream::is_create_stream(&b));
    }

    #[test]
    fn differential() {
        let a = RtmpCreateStream::body(1.0);
        let b = RtmpCreateStream::body(2.0);
        assert_eq!(a.len(), b.len());
        assert_ne!(a[14..], b[14..]);
    }
}
