//! Safe pure OutputDelayQueue (OBS obs-output-delay.c).
//!
//! Parsed from libobs/obs-output-delay.c.
//! Holds EncoderPackets for a set number of nanoseconds
//! before releasing them. Messages are PACKET, START, STOP.
//! Packets older than the delay are released. No unwraps.
//!
//! Differential: a packet pushed at t=0 is released after
//! the delay has elapsed.

use crate::encoder_packet::EncoderPacket;

/// Delay message kinds from OBS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayMsg {
    Packet,
    Start,
    Stop,
}

/// One entry in the delay queue.
#[derive(Clone, Debug)]
pub struct DelayData {
    pub msg: DelayMsg,
    pub ts: u64,
    pub packet: Option<EncoderPacket>,
}

#[derive(Clone, Debug)]
pub struct OutputDelayQueue {
    delay_ns: u64,
    data: Vec<DelayData>,
    active: bool,
}

impl OutputDelayQueue {
    /// Create with a delay in nanoseconds. Zero means no delay.
    pub fn new(delay_ns: u64) -> Self {
        Self {
            delay_ns,
            data: Vec::new(),
            active: delay_ns > 0,
        }
    }

    pub fn delay_ns(&self) -> u64 {
        self.delay_ns
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Push a packet with the current timestamp.
    pub fn push_packet(&mut self, packet: EncoderPacket, now: u64) {
        self.data.push(DelayData {
            msg: DelayMsg::Packet,
            ts: now,
            packet: Some(packet),
        });
    }

    /// Push a start message.
    pub fn push_start(&mut self, now: u64) {
        self.data.push(DelayData {
            msg: DelayMsg::Start,
            ts: now,
            packet: None,
        });
    }

    /// Push a stop message.
    pub fn push_stop(&mut self, now: u64) {
        self.data.push(DelayData {
            msg: DelayMsg::Stop,
            ts: now,
            packet: None,
        });
    }

    /// Release entries whose age exceeds the delay.
    /// Returns the released entries in order.
    /// Matching OBS pop_packet: elapsed = now - ts; release if elapsed > delay.
    pub fn pop_due(&mut self, now: u64) -> Vec<DelayData> {
        let mut released = Vec::new();
        while let Some(front) = self.data.first() {
            let elapsed = now.saturating_sub(front.ts);
            if elapsed > self.delay_ns {
                released.push(self.data.remove(0));
            } else {
                break;
            }
        }
        released
    }

    /// Clear the queue. Matching OBS obs_output_cleanup_delay.
    pub fn cleanup(&mut self) {
        self.data.clear();
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;

    #[test]
    fn packet_held_until_delay() {
        let mut q = OutputDelayQueue::new(1_000_000_000); // 1 second
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]);
        q.push_packet(pkt, 0);
        assert!(q.pop_due(500_000_000).is_empty());
        let due = q.pop_due(1_000_000_001);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].msg, DelayMsg::Packet);
        assert!(due[0].packet.is_some());
    }

    #[test]
    fn zero_delay_releases_immediately() {
        let mut q = OutputDelayQueue::new(0);
        let pkt = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![3]);
        q.push_packet(pkt, 100);
        let due = q.pop_due(101);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn start_stop_messages() {
        let mut q = OutputDelayQueue::new(100);
        q.push_start(0);
        q.push_stop(0);
        let due = q.pop_due(101);
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].msg, DelayMsg::Start);
        assert_eq!(due[1].msg, DelayMsg::Stop);
    }

    #[test]
    fn cleanup_clears() {
        let mut q = OutputDelayQueue::new(1000);
        q.push_packet(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]), 0);
        q.cleanup();
        assert!(q.is_empty());
        assert!(!q.is_active());
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(100);
        let mut b = OutputDelayQueue::new(1000);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        a.push_packet(pkt.clone(), 0);
        b.push_packet(pkt, 0);
        assert!(!a.pop_due(101).is_empty());
        assert!(b.pop_due(101).is_empty());
    }
}
