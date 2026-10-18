//! Safe pure ReconnectClear (successful write → reset).
//!
//! If a delayed write produces packets, the reconnect
//! counter is cleared. An empty write leaves it alone.
//! No unwraps.
//!
//! Differential: a successful write resets; an empty one does not.

use crate::delay_file::DelayFile;
use crate::encoder_packet::EncoderPacket;
use crate::output_delay_queue::OutputDelayQueue;
use crate::packet_file::PacketFile;
use crate::reconnect::Reconnect;

pub struct ReconnectClear;

impl ReconnectClear {
    /// Push a packet through the delay file.
    /// If the file contains packets, clear the reconnect state.
    /// Returns the number of packets written.
    pub fn push(
        queue: &mut OutputDelayQueue,
        reconnect: &mut Reconnect,
        packet: EncoderPacket,
        now: u64,
    ) -> usize {
        let buf = DelayFile::push_and_write(queue, packet, now);
        let count = PacketFile::count(&buf).unwrap_or(0);
        if count > 0 {
            reconnect.success();
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn success_clears() {
        let mut q = OutputDelayQueue::new(100);
        let mut r = Reconnect::new();
        r.start();
        assert_eq!(r.attempts(), 1);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        ReconnectClear::push(&mut q, &mut r, pkt, 0);
        assert_eq!(r.attempts(), 1);
        let pkt2 = EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]);
        let n = ReconnectClear::push(&mut q, &mut r, pkt2, 101);
        assert!(n >= 1);
        assert_eq!(r.attempts(), 0);
    }

    #[test]
    fn empty_keeps() {
        let mut q = OutputDelayQueue::new(1000);
        let mut r = Reconnect::new();
        r.start();
        let pkt = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![1]);
        assert_eq!(ReconnectClear::push(&mut q, &mut r, pkt, 0), 0);
        assert_eq!(r.attempts(), 1);
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(50);
        let mut b = OutputDelayQueue::new(50);
        let mut ra = Reconnect::new();
        let mut rb = Reconnect::new();
        ra.start();
        rb.start();
        ReconnectClear::push(
            &mut a,
            &mut ra,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        ReconnectClear::push(
            &mut b,
            &mut rb,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        ReconnectClear::push(
            &mut b,
            &mut rb,
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]),
            51,
        );
        assert_eq!(ra.attempts(), 1);
        assert_eq!(rb.attempts(), 0);
    }
}
