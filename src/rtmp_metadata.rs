//! Safe pure RtmpMetadata (OBS onMetaData / setDataFrame).
//!
//! Builds a minimal AMF0 onMetaData object with width, height,
//! and framerate. Sent after publish and before video.
//! No unwraps.
//!
//! Differential: a larger width produces a different body.

pub struct RtmpMetadata;

impl RtmpMetadata {
    /// Build the AMF0 onMetaData body.
    pub fn body(width: u32, height: u32, fps: f64) -> Vec<u8> {
        let mut out = Vec::new();
        // String "onMetaData"
        out.push(0x02);
        let name = b"onMetaData";
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
        out.extend_from_slice(name);
        // Object
        out.push(0x03);
        // width
        Self::number_prop(&mut out, b"width", width as f64);
        // height
        Self::number_prop(&mut out, b"height", height as f64);
        // framerate
        Self::number_prop(&mut out, b"framerate", fps);
        // Object end
        out.extend_from_slice(&[0x00, 0x00, 0x09]);
        out
    }

    fn number_prop(out: &mut Vec<u8>, key: &[u8], value: f64) {
        out.extend_from_slice(&(key.len() as u16).to_be_bytes());
        out.extend_from_slice(key);
        out.push(0x00);
        out.extend_from_slice(&value.to_be_bytes());
    }

    /// True if the body starts with the onMetaData string.
    pub fn is_metadata(buf: &[u8]) -> bool {
        buf.len() > 13 && buf[0] == 0x02 && &buf[3..13] == b"onMetaData"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let b = RtmpMetadata::body(1280, 720, 30.0);
        assert!(RtmpMetadata::is_metadata(&b));
        assert!(b.len() > 40);
    }

    #[test]
    fn zero_still_metadata() {
        let b = RtmpMetadata::body(0, 0, 0.0);
        assert!(RtmpMetadata::is_metadata(&b));
    }

    #[test]
    fn differential() {
        let a = RtmpMetadata::body(640, 360, 30.0);
        let b = RtmpMetadata::body(1920, 1080, 60.0);
        assert_ne!(a, b);
        assert_eq!(a.len(), b.len());
    }
}
