//! Safe pure CaptionAttach (caption → packet pts).
//!
//! Associates a caption line with a packet's pts.
//! Empty text is rejected. No unwraps.
//!
//! Differential: a later pts produces a later caption.

use crate::caption_track::{CaptionError, CaptionTrack};
use crate::encoder_packet::EncoderPacket;

pub struct CaptionAttach;

impl CaptionAttach {
    /// Push a caption at the packet's pts.
    /// Returns error if the text is empty or too long.
    pub fn attach(
        track: &mut CaptionTrack,
        packet: &EncoderPacket,
        text: &str,
        duration: f64,
    ) -> Result<(), CaptionError> {
        track.set_timestamp(packet.pts as f64);
        track.push(text, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn attach_sets_ts() {
        let mut t = CaptionTrack::new();
        let pkt = EncoderPacket::new(EncoderType::Video, 100, 100, vec![1]);
        CaptionAttach::attach(&mut t, &pkt, "hello", 2.0).unwrap();
        assert_eq!(t.timestamp(), 100.0);
        assert_eq!(t.len(), 1);
        assert_eq!(t.line(0).unwrap().text(), "hello");
    }

    #[test]
    fn empty_rejected() {
        let mut t = CaptionTrack::new();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        assert!(CaptionAttach::attach(&mut t, &pkt, "", 1.0).is_err());
    }

    #[test]
    fn differential() {
        let mut a = CaptionTrack::new();
        let mut b = CaptionTrack::new();
        let early = EncoderPacket::new(EncoderType::Video, 10, 10, vec![1]);
        let late = EncoderPacket::new(EncoderType::Video, 50, 50, vec![1]);
        CaptionAttach::attach(&mut a, &early, "a", 1.0).unwrap();
        CaptionAttach::attach(&mut b, &late, "b", 1.0).unwrap();
        assert!(b.timestamp() > a.timestamp());
    }
}
