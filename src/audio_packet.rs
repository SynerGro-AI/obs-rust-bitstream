//! Safe pure AudioPacket (samples → EncoderPacket).
//!
//! Pulls samples from an AudioSourceList and wraps them
//! as an EncoderPacket of type Audio. Empty or missing
//! pulls return None. No unwraps.
//!
//! Differential: two pulls produce different packets.

use crate::audio_source_list::AudioSourceList;
use crate::encoder_context::EncoderType;
use crate::encoder_packet::EncoderPacket;

pub struct AudioPacket;

impl AudioPacket {
    /// Pull frames and wrap them as an audio EncoderPacket.
    /// pts and dts are set to the supplied timestamp.
    /// Returns None if the list yields nothing.
    pub fn from_list(
        list: &mut AudioSourceList,
        frames: usize,
        pts: i64,
    ) -> Option<EncoderPacket> {
        let samples = list.samples(frames)?;
        if samples.data.is_empty() {
            return None;
        }
        let mut bytes = Vec::with_capacity(samples.data.len() * 4);
        for s in &samples.data {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        Some(EncoderPacket::new(EncoderType::Audio, pts, pts, bytes).track(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;

    #[test]
    fn produces_packet() {
        let mut list = AudioSourceList::new();
        list.add("tone", AudioSource::synthetic(48000, 1));
        let pkt = AudioPacket::from_list(&mut list, 8, 100).unwrap();
        assert_eq!(pkt.kind, EncoderType::Audio);
        assert_eq!(pkt.pts, 100);
        assert_eq!(pkt.data.len(), 32);
    }

    #[test]
    fn empty_is_none() {
        let mut list = AudioSourceList::new();
        assert!(AudioPacket::from_list(&mut list, 8, 0).is_none());
    }

    #[test]
    fn differential() {
        let mut list = AudioSourceList::new();
        list.add("tone", AudioSource::synthetic(48000, 1));
        let a = AudioPacket::from_list(&mut list, 4, 0).unwrap();
        let b = AudioPacket::from_list(&mut list, 4, 1).unwrap();
        assert_ne!(a.data, b.data);
        assert_ne!(a.pts, b.pts);
    }
}
