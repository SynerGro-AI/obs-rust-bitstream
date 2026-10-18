//! Safe pure OutputFeed (interleave → output).
//!
//! Drains packets from an InterleaveQueue into a running
//! OutputInfo. Stopped outputs accept nothing. No unwraps.
//!
//! Differential: a running output counts bytes; a stopped one does not.

use crate::encoder_packet::InterleaveQueue;
use crate::output_info::OutputInfo;

pub struct OutputFeed;

impl OutputFeed {
    /// Drain the queue into the output.
    /// Returns the number of packets accepted.
    /// A stopped output accepts zero.
    pub fn drain(queue: &mut InterleaveQueue, output: &mut OutputInfo) -> usize {
        if !output.is_running() {
            return 0;
        }
        let mut n = 0;
        while let Some(pkt) = queue.pop() {
            if output.add_bytes(pkt.data.len() as u64) {
                n += 1;
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;
    use crate::output_info::{OutputInfo, OUTPUT_AV};

    #[test]
    fn running_counts() {
        let mut q = InterleaveQueue::new();
        q.push(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2, 3]));
        q.push(EncoderPacket::new(EncoderType::Audio, 1, 1, vec![4]));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        out.start();
        let n = OutputFeed::drain(&mut q, &mut out);
        assert_eq!(n, 2);
        assert_eq!(out.bytes(), 4);
        assert!(q.is_empty());
    }

    #[test]
    fn stopped_is_zero() {
        let mut q = InterleaveQueue::new();
        q.push(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        assert_eq!(OutputFeed::drain(&mut q, &mut out), 0);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn differential() {
        let mut q = InterleaveQueue::new();
        q.push(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]));
        let mut a = OutputInfo::new("a", "A", OUTPUT_AV).unwrap();
        a.start();
        let mut b = OutputInfo::new("b", "B", OUTPUT_AV).unwrap();
        OutputFeed::drain(&mut q, &mut a);
        let mut q2 = InterleaveQueue::new();
        q2.push(EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]));
        OutputFeed::drain(&mut q2, &mut b);
        assert!(a.bytes() > b.bytes());
    }
}
