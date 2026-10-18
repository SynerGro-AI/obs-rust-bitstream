//! Safe pure RtmpStatus (RTMP stream report).
//!
//! One line of active, packets, and bytes.
//! No unwraps.
//!
//! Differential: more packets produce a higher count.

use crate::rtmp_stream::RtmpStream;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RtmpStatus {
    pub active: bool,
    pub packets: usize,
    pub bytes: u64,
    pub url: String,
}

impl RtmpStatus {
    /// Build from an RtmpStream.
    pub fn from(stream: &RtmpStream) -> Self {
        Self {
            active: stream.is_active(),
            packets: stream.packet_count(),
            bytes: stream.bytes(),
            url: stream.url().to_string(),
        }
    }

    /// One line: "active=true/false packets=N bytes=N"
    pub fn one_line(&self) -> String {
        format!(
            "active={} packets={} bytes={}",
            self.active, self.packets, self.bytes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;
    use crate::rtmp_feed::RtmpFeed;

    #[test]
    fn reports() {
        let packets = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2])];
        let s = RtmpFeed::feed("live.twitch.tv", "abc", &packets).unwrap();
        let st = RtmpStatus::from(&s);
        assert!(st.active);
        assert_eq!(st.packets, 1);
        assert_eq!(st.bytes, 2);
        assert!(st.one_line().contains("active=true"));
    }

    #[test]
    fn inactive() {
        let s = RtmpStream::new("rtmp://x").unwrap();
        let st = RtmpStatus::from(&s);
        assert!(!st.active);
        assert_eq!(st.packets, 0);
    }

    #[test]
    fn differential() {
        let one = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let two = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]),
        ];
        let a = RtmpStatus::from(&RtmpFeed::feed("a", "k", &one).unwrap());
        let b = RtmpStatus::from(&RtmpFeed::feed("b", "k", &two).unwrap());
        assert!(b.packets > a.packets);
    }
}
