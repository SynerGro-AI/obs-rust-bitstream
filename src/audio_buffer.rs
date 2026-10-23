//! Safe pure audio buffer (OBS audio data core).
//!
//! The C version uses plane pointers and frame counts.
//! The safe path owns a Vec of planes sized from AudioFormat and
//! SpeakerLayout. Zero frames or unknown formats return None.
//! No raw pointers, no unwraps.
//!
//! Differential: total bytes match audio_size.

use crate::audio_format::{audio_size, AudioFormat, SpeakerLayout};
#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub planes: Vec<Vec<u8>>,
    pub frames: u32,
    pub format: AudioFormat,
    pub speakers: SpeakerLayout,
}

impl AudioBuffer {
    pub fn new(format: AudioFormat, speakers: SpeakerLayout, frames: u32) -> Option<Self> {
        let total = audio_size(format, speakers, frames)?;
        let plane_count = format.planes(speakers);
        if plane_count == 0 {
            return None;
        }
        let per_plane = total / plane_count;
        let mut planes = Vec::with_capacity(plane_count);
        for _ in 0..plane_count {
            planes.push(vec![0u8; per_plane]);
        }
        Some(Self {
            planes,
            frames,
            format,
            speakers,
        })
    }

    pub fn total_bytes(&self) -> usize {
        self.planes.iter().map(|p| p.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_s16() {
        let b = AudioBuffer::new(AudioFormat::S16, SpeakerLayout::Stereo, 1024).unwrap();
        assert_eq!(b.planes.len(), 1); // packed
        assert_eq!(b.total_bytes(), 2 * 2 * 1024);
    }

    #[test]
    fn planar() {
        let b = AudioBuffer::new(AudioFormat::F32Planar, SpeakerLayout::Stereo, 100).unwrap();
        assert_eq!(b.planes.len(), 2);
        assert_eq!(b.total_bytes(), 4 * 2 * 100);
    }

    #[test]
    fn zero_safe() {
        assert!(AudioBuffer::new(AudioFormat::S16, SpeakerLayout::Mono, 0).is_none());
        assert!(AudioBuffer::new(AudioFormat::Unknown, SpeakerLayout::Stereo, 10).is_none());
    }

    #[test]
    fn differential() {
        let b = AudioBuffer::new(AudioFormat::S16, SpeakerLayout::Mono, 48).unwrap();
        let expected = audio_size(AudioFormat::S16, SpeakerLayout::Mono, 48).unwrap();
        assert_equal(b.total_bytes(), expected, "audio buffer size");
    }
}
