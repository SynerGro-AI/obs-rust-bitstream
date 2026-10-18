//! Safe pure PauseDelay (pause gates the delay release).
//!
//! If the output is paused, packets stay in the delay queue.
//! If not paused, due packets are released to callbacks.
//! No unwraps.
//!
//! Differential: a paused output holds; an unpaused one releases.

use crate::delay_callback::DelayCallback;
use crate::encoder_packet::EncoderPacket;
use crate::output_delay_queue::OutputDelayQueue;
use crate::output_pause::OutputPause;
use crate::packet_callback::PacketCallbackList;

pub struct PauseDelay;

impl PauseDelay {
    /// Push a packet. If paused, it stays held.
    /// If not paused, due packets notify callbacks.
    /// Returns the number of callbacks that accepted.
    pub fn push(
        pause: &OutputPause,
        queue: &mut OutputDelayQueue,
        callbacks: &PacketCallbackList,
        packet: EncoderPacket,
        now: u64,
    ) -> usize {
        if pause.is_paused() {
            queue.push_packet(packet, now);
            return 0;
        }
        DelayCallback::push_and_release(queue, callbacks, packet, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};
    use crate::output_pause::{OutputPause, OUTPUT_CAN_PAUSE};

    fn pausable() -> OutputInfo {
        OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED | OUTPUT_CAN_PAUSE).unwrap()
    }

    #[test]
    fn paused_holds() {
        let out = pausable();
        let mut pause = OutputPause::new();
        pause.start(&out, 0).unwrap();
        let mut q = OutputDelayQueue::new(0);
        let mut cbs = PacketCallbackList::new();
        cbs.add("mux").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let n = PauseDelay::push(&pause, &mut q, &cbs, pkt, 1);
        assert_eq!(n, 0);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn unpaused_releases() {
        let pause = OutputPause::new();
        let mut q = OutputDelayQueue::new(0);
        let mut cbs = PacketCallbackList::new();
        cbs.add("mux").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let n = PauseDelay::push(&pause, &mut q, &cbs, pkt, 1);
        assert!(n >= 1);
    }

    #[test]
    fn differential() {
        let out = pausable();
        let mut paused = OutputPause::new();
        paused.start(&out, 0).unwrap();
        let unpaused = OutputPause::new();
        let mut q1 = OutputDelayQueue::new(0);
        let mut q2 = OutputDelayQueue::new(0);
        let mut cbs = PacketCallbackList::new();
        cbs.add("mux").unwrap();
        let a = PauseDelay::push(
            &paused,
            &mut q1,
            &cbs,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            1,
        );
        let b = PauseDelay::push(
            &unpaused,
            &mut q2,
            &cbs,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            1,
        );
        assert!(b > a);
    }
}
