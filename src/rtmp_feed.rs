//! Safe pure RtmpFeed (packets → RTMP stream).
//!
//! Connects an RtmpStream and pushes packets.
//! Inactive streams reject. No unwraps.
//!
//! Differential: more packets produce more bytes.

use crate::encoder_packet::EncoderPacket;
use crate::rtmp_stream::{RtmpError, RtmpStream};
use crate::rtmp_start::RtmpStart;

pub struct RtmpFeed;

impl RtmpFeed {
    /// Start and push all packets. Returns the connected stream.
    pub fn feed(name: &str, key: &str, packets: &[EncoderPacket]) -> Result<RtmpStream, RtmpError> {
        let mut stream = RtmpStart::start(name, key)?;
        for pkt in packets {
            stream.push(pkt)?;
        }
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn feeds() {
        let packets = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]),
            EncoderPacket::new(EncoderType::Audio, 1, 1, vec![3]),
        ];
        let s = RtmpFeed::feed("live.twitch.tv", "abc", &packets).unwrap();
        assert!(s.is_active());
        assert!(s.got_first_packet());
        assert_eq!(s.packet_count(), 2);
        assert_eq!(s.bytes(), 3);
    }

    #[test]
    fn empty_rejected() {
        assert!(RtmpFeed::feed("", "abc", &[]).is_err());
    }

    #[test]
    fn differential() {
        let one = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let two = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2, 3]),
        ];
        let a = RtmpFeed::feed("a", "k", &one).unwrap();
        let b = RtmpFeed::feed("b", "k", &two).unwrap();
        assert!(b.bytes() > a.bytes());
    }
}
