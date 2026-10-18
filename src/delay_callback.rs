//! Safe pure DelayCallback (delay queue → packet callbacks).
//!
//! Pushes packets into an OutputDelayQueue and, when they
//! become due, invokes a PacketCallbackList. No unwraps.
//!
//! Differential: a due packet notifies; an early one does not.

use crate::encoder_packet::EncoderPacket;
use crate::output_delay_queue::OutputDelayQueue;
use crate::packet_callback::PacketCallbackList;

pub struct DelayCallback;

impl DelayCallback {
    /// Push a packet and release any that are due.
    /// Returns the number of callbacks that accepted due packets.
    pub fn push_and_release(
        queue: &mut OutputDelayQueue,
        callbacks: &PacketCallbackList,
        packet: EncoderPacket,
        now: u64,
    ) -> usize {
        queue.push_packet(packet, now);
        let due = queue.pop_due(now);
        let mut accepted = 0;
        for entry in due {
            if let Some(pkt) = entry.packet {
                accepted += callbacks.invoke(&pkt);
            }
        }
        accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn due_notifies() {
        let mut q = OutputDelayQueue::new(100);
        let mut cbs = PacketCallbackList::new();
        cbs.add("mux").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let n = DelayCallback::push_and_release(&mut q, &cbs, pkt, 0);
        assert_eq!(n, 0);
        let pkt2 = EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]);
        let n2 = DelayCallback::push_and_release(&mut q, &cbs, pkt2, 101);
        assert!(n2 >= 1);
    }

    #[test]
    fn empty_callbacks() {
        let mut q = OutputDelayQueue::new(0);
        let cbs = PacketCallbackList::new();
        let pkt = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![1]);
        assert_eq!(DelayCallback::push_and_release(&mut q, &cbs, pkt, 1), 0);
    }

    #[test]
    fn differential() {
        let mut q = OutputDelayQueue::new(50);
        let mut cbs = PacketCallbackList::new();
        cbs.add("a").unwrap();
        cbs.add("b").unwrap();
        let early = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let late = EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]);
        let a = DelayCallback::push_and_release(&mut q, &cbs, early, 0);
        let b = DelayCallback::push_and_release(&mut q, &cbs, late, 51);
        assert!(b > a);
    }
}
