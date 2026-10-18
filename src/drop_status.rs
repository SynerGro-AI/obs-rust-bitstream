//! Safe pure DropStatus (dropped versus kept).
//!
//! Counts how many packets were dropped and how many
//! were kept under a threshold. No unwraps.
//!
//! Differential: a higher threshold drops more.

use crate::encoder_packet::EncoderPacket;
use crate::packet_drop::PacketDrop;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropStatus {
    pub dropped: usize,
    pub kept: usize,
}

impl DropStatus {
    /// Count dropped and kept under the threshold.
    pub fn from(packets: &[EncoderPacket], threshold: i32) -> Self {
        let mut dropped = 0;
        let mut kept = 0;
        for pkt in packets {
            if PacketDrop::should_drop(pkt, threshold) {
                dropped += 1;
            } else {
                kept += 1;
            }
        }
        Self { dropped, kept }
    }

    /// One line: "dropped=N kept=N"
    pub fn one_line(&self) -> String {
        format!("dropped={} kept={}", self.dropped, self.kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn counts() {
        let packets = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1),
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]).priority(5),
        ];
        let s = DropStatus::from(&packets, 3);
        assert_eq!(s.dropped, 1);
        assert_eq!(s.kept, 1);
        assert!(s.one_line().contains("dropped=1"));
    }

    #[test]
    fn empty_is_zero() {
        let s = DropStatus::from(&[], 0);
        assert_eq!(s.dropped, 0);
        assert_eq!(s.kept, 0);
    }

    #[test]
    fn differential() {
        let packets = vec![
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1),
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]).priority(2),
        ];
        let a = DropStatus::from(&packets, 0);
        let b = DropStatus::from(&packets, 3);
        assert!(b.dropped > a.dropped);
    }
}
