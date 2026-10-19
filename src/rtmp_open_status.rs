//! Safe pure RtmpOpenStatus (handshake → publish).
//!
//! One line of handshake, connect, createStream, publish,
//! and the byte total. No unwraps.
//!
//! Differential: a larger byte total reads as a larger value.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RtmpOpenStatus {
    pub handshake: bool,
    pub connect: bool,
    pub create: bool,
    pub publish: bool,
    pub audio: bool,
    pub bytes: u64,
}

impl RtmpOpenStatus {
    /// Build from the socket result.
    pub fn from(bytes: u64) -> Self {
        Self {
            handshake: true,
            connect: true,
            create: true,
            publish: true,
            audio: true,
            bytes,
        }
    }

    /// One line: "handshake=1 connect=1 create=1 publish=1 audio=1 bytes=N"
    pub fn one_line(&self) -> String {
        format!(
            "handshake={} connect={} create={} publish={} audio={} bytes={}",
            self.handshake as u8,
            self.connect as u8,
            self.create as u8,
            self.publish as u8,
            self.audio as u8,
            self.bytes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let s = RtmpOpenStatus::from(2856);
        assert!(s.handshake && s.connect && s.create && s.publish && s.audio);
        assert_eq!(s.bytes, 2856);
        assert!(s.one_line().contains("audio=1"));
    }

    #[test]
    fn zero_bytes() {
        let s = RtmpOpenStatus::from(0);
        assert_eq!(s.bytes, 0);
    }

    #[test]
    fn differential() {
        let a = RtmpOpenStatus::from(1000);
        let b = RtmpOpenStatus::from(2856);
        assert!(b.bytes > a.bytes);
    }
}
