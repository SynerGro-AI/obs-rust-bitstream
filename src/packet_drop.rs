//! Safe pure PacketDrop (priority gate).
//!
//! Decides whether a packet should be dropped based on
//! its priority and the current drop threshold.
//! A packet is kept if its priority is at or above the threshold.
//! No unwraps.
//!
//! Differential: a higher threshold drops more packets.

use crate::encoder_packet::EncoderPacket;

pub struct PacketDrop;

impl PacketDrop {
    /// True if the packet should be dropped.
    /// Dropped when priority is below the threshold.
    pub fn should_drop(packet: &EncoderPacket, threshold: i32) -> bool {
        packet.priority < threshold
    }

    /// True if the next packet must meet the drop priority.
    /// Used after a drop to require a higher-priority continuation.
    pub fn requires_next(packet: &EncoderPacket) -> i32 {
        packet.drop_priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn keeps_high() {
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(5);
        assert!(!PacketDrop::should_drop(&pkt, 3));
    }

    #[test]
    fn drops_low() {
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1);
        assert!(PacketDrop::should_drop(&pkt, 3));
    }

    #[test]
    fn requires_next() {
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).drop_priority(4);
        assert_eq!(PacketDrop::requires_next(&pkt), 4);
    }

    #[test]
    fn differential() {
        let low = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1);
        let high = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(5);
        assert!(PacketDrop::should_drop(&low, 3));
        assert!(!PacketDrop::should_drop(&high, 3));
    }
}
