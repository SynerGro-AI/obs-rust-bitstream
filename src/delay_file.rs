//! Safe pure DelayFile (delay queue → packet file).
//!
//! Releases due packets and writes their payloads to a
//! PacketFile buffer. Early packets stay held. No unwraps.
//!
//! Differential: a due packet produces a larger file.

use crate::encoder_packet::EncoderPacket;
use crate::output_delay_queue::OutputDelayQueue;
use crate::packet_file::PacketFile;

pub struct DelayFile;

impl DelayFile {
    /// Push a packet and write any due payloads.
    /// Returns the file buffer. Held packets are not written.
    pub fn push_and_write(
        queue: &mut OutputDelayQueue,
        packet: EncoderPacket,
        now: u64,
    ) -> Vec<u8> {
        queue.push_packet(packet, now);
        let due = queue.pop_due(now);
        let mut packets = Vec::new();
        for entry in due {
            if let Some(pkt) = entry.packet {
                packets.push(pkt);
            }
        }
        PacketFile::write(&packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn due_writes() {
        let mut q = OutputDelayQueue::new(100);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]);
        let early = DelayFile::push_and_write(&mut q, pkt, 0);
        assert_eq!(PacketFile::count(&early), Some(0));
        let pkt2 = EncoderPacket::new(EncoderType::Video, 1, 1, vec![3]);
        let late = DelayFile::push_and_write(&mut q, pkt2, 101);
        assert!(PacketFile::count(&late).unwrap() >= 1);
    }

    #[test]
    fn empty_is_header() {
        let mut q = OutputDelayQueue::new(0);
        let pkt = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![1]);
        let buf = DelayFile::push_and_write(&mut q, pkt, 0);
        assert_eq!(PacketFile::count(&buf), Some(0));
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(50);
        let mut b = OutputDelayQueue::new(50);
        let early = DelayFile::push_and_write(
            &mut a,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        DelayFile::push_and_write(
            &mut b,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        let late = DelayFile::push_and_write(
            &mut b,
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]),
            51,
        );
        assert!(late.len() > early.len());
    }
}
