//! Safe pure KeyframeMark (packet → keyframe group).
//!
//! If a packet is a keyframe, mark the track as seen.
//! Non-keyframes are ignored. Out-of-range tracks are ignored.
//! No unwraps.
//!
//! Differential: a keyframe increases the done count.

use crate::encoder_packet::EncoderPacket;
use crate::keyframe_group::KeyframeGroup;

pub struct KeyframeMark;

impl KeyframeMark {
    /// Mark the track if the packet is a keyframe.
    /// Returns true if the group became complete.
    pub fn mark(group: &mut KeyframeGroup, packet: &EncoderPacket, track: usize) -> bool {
        if packet.keyframe {
            group.mark_seen(track);
        }
        group.is_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn keyframe_marks() {
        let mut g = KeyframeGroup::new(1, 0, 1);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).keyframe();
        assert!(KeyframeMark::mark(&mut g, &pkt, 0));
        assert!(g.is_complete());
    }

    #[test]
    fn non_keyframe_ignored() {
        let mut g = KeyframeGroup::new(1, 0, 1);
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        assert!(!KeyframeMark::mark(&mut g, &pkt, 0));
        assert!(!g.is_complete());
    }

    #[test]
    fn differential() {
        let mut a = KeyframeGroup::new(1, 0, 2);
        let mut b = KeyframeGroup::new(2, 0, 2);
        let kf = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]).keyframe();
        KeyframeMark::mark(&mut a, &kf, 0);
        KeyframeMark::mark(&mut b, &kf, 0);
        KeyframeMark::mark(&mut b, &kf, 1);
        assert!(b.done_count() > a.done_count());
    }
}
