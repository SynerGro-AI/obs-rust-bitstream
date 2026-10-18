//! Safe pure RtmpConnect (OBS librtmp connect command).
//!
//! Builds a minimal AMF0 connect packet body.
//! Command name "connect", transaction id 1, app object.
//! No unwraps.
//!
//! Differential: a different app name produces a larger body.

pub struct RtmpConnect;

impl RtmpConnect {
    /// Build the AMF0 connect body for `app`.
    /// Returns the command bytes (not yet chunked).
    pub fn body(app: &str) -> Vec<u8> {
        let mut out = Vec::new();
        // String "connect"
        out.push(0x02);
        let name = b"connect";
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
        out.extend_from_slice(name);
        // Number 1.0 (transaction id)
        out.push(0x00);
        out.extend_from_slice(&1.0f64.to_be_bytes());
        // Object start
        out.push(0x03);
        // app property
        let key = b"app";
        out.extend_from_slice(&(key.len() as u16).to_be_bytes());
        out.extend_from_slice(key);
        out.push(0x02);
        out.extend_from_slice(&(app.len() as u16).to_be_bytes());
        out.extend_from_slice(app.as_bytes());
        // Object end
        out.extend_from_slice(&[0x00, 0x00, 0x09]);
        out
    }

    /// True if the body starts with the connect string marker.
    pub fn is_connect(buf: &[u8]) -> bool {
        buf.len() > 10 && buf[0] == 0x02 && &buf[3..10] == b"connect"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let b = RtmpConnect::body("live");
        assert!(RtmpConnect::is_connect(&b));
        assert!(b.len() > 20);
    }

    #[test]
    fn empty_app_still_connect() {
        let b = RtmpConnect::body("");
        assert!(RtmpConnect::is_connect(&b));
    }

    #[test]
    fn differential() {
        let a = RtmpConnect::body("a");
        let b = RtmpConnect::body("live");
        assert!(b.len() > a.len());
    }
}
