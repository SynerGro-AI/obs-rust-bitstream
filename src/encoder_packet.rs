//! Safe pure EncoderPacket (OBS encoder_packet core).
//!
//! Parsed from libobs/obs-encoder.h.
//! Holds data, pts, dts, timebase, type, and keyframe.
//! An InterleaveQueue orders packets by dts so video and
//! audio stay monotonic. No unwraps.
//!
//! Differential: two packets with different dts sort.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderType {
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncoderPacket {
    pub data: Vec<u8>,
    pub pts: i64,
    pub dts: i64,
    pub timebase_num: i32,
    pub timebase_den: i32,
    pub kind: EncoderType,
    pub keyframe: bool,
    pub track_idx: usize,
}

impl EncoderPacket {
    pub fn new(kind: EncoderType, pts: i64, dts: i64, data: Vec<u8>) -> Self {
        Self {
            data,
            pts,
            dts,
            timebase_num: 1,
            timebase_den: 1_000_000,
            kind,
            keyframe: false,
            track_idx: 0,
        }
    }

    pub fn keyframe(mut self) -> Self {
        self.keyframe = true;
        self
    }

    pub fn track(mut self, idx: usize) -> Self {
        self.track_idx = idx;
        self
    }
}

/// Orders packets by dts, matching the OBS interleaved_packets idea.
#[derive(Clone, Debug, Default)]
pub struct InterleaveQueue {
    packets: Vec<EncoderPacket>,
}

impl InterleaveQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, packet: EncoderPacket) {
        self.packets.push(packet);
        self.packets.sort_by_key(|p| p.dts);
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// Pop the packet with the lowest dts.
    pub fn pop(&mut self) -> Option<EncoderPacket> {
        if self.packets.is_empty() {
            None
        } else {
            Some(self.packets.remove(0))
        }
    }

    /// True if the queue is monotonic by dts.
    pub fn is_monotonic(&self) -> bool {
        self.packets.windows(2).all(|w| w[0].dts <= w[1].dts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_fields() {
        let p = EncoderPacket::new(EncoderType::Video, 1, 0, vec![1, 2, 3]).keyframe();
        assert_eq!(p.kind, EncoderType::Video);
        assert!(p.keyframe);
        assert_eq!(p.data.len(), 3);
    }

    #[test]
    fn interleave_orders_by_dts() {
        let mut q = InterleaveQueue::new();
        q.push(EncoderPacket::new(EncoderType::Audio, 2, 200, vec![2]));
        q.push(EncoderPacket::new(EncoderType::Video, 1, 100, vec![1]));
        assert!(q.is_monotonic());
        let first = q.pop().unwrap();
        assert_eq!(first.dts, 100);
        assert_eq!(first.kind, EncoderType::Video);
    }

    #[test]
    fn empty_pop() {
        let mut q = InterleaveQueue::new();
        assert!(q.pop().is_none());
    }

    #[test]
    fn differential() {
        let mut q = InterleaveQueue::new();
        q.push(EncoderPacket::new(EncoderType::Video, 0, 10, vec![1]));
        q.push(EncoderPacket::new(EncoderType::Audio, 0, 20, vec![2]));
        let a = q.pop().unwrap();
        let b = q.pop().unwrap();
        assert!(a.dts < b.dts);
    }
}
