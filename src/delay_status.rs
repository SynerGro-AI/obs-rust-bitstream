//! Safe pure DelayStatus (delay + pause + reconnect report).
//!
//! One line of the current delay queue length, pause state,
//! and reconnect attempt count. No unwraps.
//!
//! Differential: a paused queue reports more held packets.

use crate::output_delay_queue::OutputDelayQueue;
use crate::output_pause::OutputPause;
use crate::reconnect::Reconnect;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelayStatus {
    pub held: usize,
    pub paused: bool,
    pub attempts: u32,
}

impl DelayStatus {
    /// Build from the current delay, pause, and reconnect state.
    pub fn from(queue: &OutputDelayQueue, pause: &OutputPause, reconnect: &Reconnect) -> Self {
        Self {
            held: queue.len(),
            paused: pause.is_paused(),
            attempts: reconnect.attempts(),
        }
    }

    /// One line: "held=N paused=true/false attempts=N"
    pub fn one_line(&self) -> String {
        format!(
            "held={} paused={} attempts={}",
            self.held, self.paused, self.attempts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};
    use crate::output_pause::{OutputPause, OUTPUT_CAN_PAUSE};

    #[test]
    fn reports() {
        let mut q = OutputDelayQueue::new(1000);
        q.push_packet(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]), 0);
        let mut pause = OutputPause::new();
        let out = OutputInfo::new(
            "rec",
            "Recording",
            OUTPUT_AV | OUTPUT_ENCODED | OUTPUT_CAN_PAUSE,
        )
        .unwrap();
        pause.start(&out, 0).unwrap();
        let mut r = Reconnect::new();
        r.start();
        let s = DelayStatus::from(&q, &pause, &r);
        assert_eq!(s.held, 1);
        assert!(s.paused);
        assert_eq!(s.attempts, 1);
        assert!(s.one_line().contains("held=1"));
    }

    #[test]
    fn empty_is_zero() {
        let q = OutputDelayQueue::new(0);
        let pause = OutputPause::new();
        let r = Reconnect::new();
        let s = DelayStatus::from(&q, &pause, &r);
        assert_eq!(s.held, 0);
        assert!(!s.paused);
        assert_eq!(s.attempts, 0);
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(100);
        let mut b = OutputDelayQueue::new(100);
        a.push_packet(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]), 0);
        b.push_packet(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]), 0);
        b.push_packet(EncoderPacket::new(EncoderType::Audio, 1, 1, vec![2]), 0);
        let pause = OutputPause::new();
        let r = Reconnect::new();
        let sa = DelayStatus::from(&a, &pause, &r);
        let sb = DelayStatus::from(&b, &pause, &r);
        assert!(sb.held > sa.held);
    }
}
