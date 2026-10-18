//! Safe pure DropFile (priority gate → packet file).
//!
//! Drops packets below the threshold. Kept packets are
//! written through the delay file. No unwraps.
//!
//! Differential: a higher threshold writes fewer packets.

use crate::delay_file::DelayFile;
use crate::encoder_packet::EncoderPacket;
use crate::output_delay_queue::OutputDelayQueue;
use crate::packet_drop::PacketDrop;
use crate::packet_file::PacketFile;

pub struct DropFile;

impl DropFile {
    /// Push a packet. If it should be dropped, write nothing.
    /// Otherwise write any due packets. Returns the count written.
    pub fn push(
        queue: &mut OutputDelayQueue,
        packet: EncoderPacket,
        threshold: i32,
        now: u64,
    ) -> usize {
        if PacketDrop::should_drop(&packet, threshold) {
            return 0;
        }
        let buf = DelayFile::push_and_write(queue, packet, now);
        PacketFile::count(&buf).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn keeps_high() {
        let mut q = OutputDelayQueue::new(0);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(5);
        DropFile::push(&mut q, pkt, 3, 0);
        let pkt2 = EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]).priority(5);
        let n = DropFile::push(&mut q, pkt2, 3, 1);
        assert!(n >= 1);
    }

    #[test]
    fn drops_low() {
        let mut q = OutputDelayQueue::new(0);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1);
        assert_eq!(DropFile::push(&mut q, pkt, 3, 1), 0);
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(0);
        let mut b = OutputDelayQueue::new(0);
        let low = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(1);
        let high = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(5);
        let na = DropFile::push(&mut a, low, 3, 1);
        DropFile::push(
            &mut b,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).priority(5),
            3,
            0,
        );
        let nb = DropFile::push(&mut b, high, 3, 1);
        assert!(nb > na);
    }
}
