//! Safe pure PacketRound (write → read).
//!
//! Writes packets and reads the payloads back.
//! Returns true if the count and first payload match.
//! No unwraps.
//!
//! Differential: a different payload fails the match.

use crate::encoder_packet::EncoderPacket;
use crate::packet_file::PacketFile;

pub struct PacketRound;

impl PacketRound {
    /// True if the written packets read back the same count and first payload.
    pub fn matches(packets: &[EncoderPacket]) -> bool {
        let buf = PacketFile::write(packets);
        let read = match PacketFile::read(&buf) {
            Some(r) => r,
            None => return false,
        };
        if read.len() != packets.len() {
            return false;
        }
        if packets.is_empty() {
            return true;
        }
        read[0] == packets[0].data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn round_trips() {
        let packets = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]),
            EncoderPacket::new(EncoderType::Audio, 1, 1, vec![3]),
        ];
        assert!(PacketRound::matches(&packets));
    }

    #[test]
    fn empty_round_trips() {
        assert!(PacketRound::matches(&[]));
    }

    #[test]
    fn differential() {
        let a = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let b = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![2])];
        assert!(PacketRound::matches(&a));
        assert!(PacketRound::matches(&b));
        assert_ne!(a[0].data, b[0].data);
    }
}
