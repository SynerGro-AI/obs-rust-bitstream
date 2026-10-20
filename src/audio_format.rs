//! Safe pure audio format helpers (OBS audio-io.h core).
//!
//! The C version provides get_audio_channels, bytes_per_channel,
//! is_planar, planes, and size calculations. The safe path returns
//! Option / 0 on unknown formats or zero frames. No unwraps.
//!
//! Differential: sizes match the mathematical channel * bytes * frames.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    Unknown,
    U8,
    S16,
    S32,
    F32,
    U8Planar,
    S16Planar,
    S32Planar,
    F32Planar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeakerLayout {
    Unknown,
    Mono,
    Stereo,
    Surround21,
    Surround40,
    Surround41,
    Surround51,
    Surround71,
}

impl SpeakerLayout {
    pub fn channels(self) -> u32 {
        match self {
            SpeakerLayout::Mono => 1,
            SpeakerLayout::Stereo => 2,
            SpeakerLayout::Surround21 => 3,
            SpeakerLayout::Surround40 => 4,
            SpeakerLayout::Surround41 => 5,
            SpeakerLayout::Surround51 => 6,
            SpeakerLayout::Surround71 => 8,
            SpeakerLayout::Unknown => 0,
        }
    }
}

impl AudioFormat {
    pub fn bytes_per_channel(self) -> usize {
        match self {
            AudioFormat::U8 | AudioFormat::U8Planar => 1,
            AudioFormat::S16 | AudioFormat::S16Planar => 2,
            AudioFormat::S32 | AudioFormat::F32 | AudioFormat::S32Planar | AudioFormat::F32Planar => 4,
            AudioFormat::Unknown => 0,
        }
    }

    pub fn is_planar(self) -> bool {
        matches!(
            self,
            AudioFormat::U8Planar | AudioFormat::S16Planar | AudioFormat::S32Planar | AudioFormat::F32Planar
        )
    }

    pub fn planes(self, speakers: SpeakerLayout) -> usize {
        if self.is_planar() {
            speakers.channels() as usize
        } else {
            1
        }
    }
}

/// Total bytes for the given frames (planar or packed).
pub fn audio_size(format: AudioFormat, speakers: SpeakerLayout, frames: u32) -> Option<usize> {
    let bpc = format.bytes_per_channel();
    let ch = speakers.channels();
    if bpc == 0 || ch == 0 || frames == 0 {
        return None;
    }
    Some(bpc * ch as usize * frames as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_s16() {
        let sz = audio_size(AudioFormat::S16, SpeakerLayout::Stereo, 1024);
        assert_eq!(sz, Some(2 * 2 * 1024));
    }

    #[test]
    fn planar() {
        assert!(AudioFormat::F32Planar.is_planar());
        assert!(!AudioFormat::F32.is_planar());
        assert_eq!(AudioFormat::S16Planar.planes(SpeakerLayout::Stereo), 2);
    }

    #[test]
    fn unknown_safe() {
        assert_eq!(audio_size(AudioFormat::Unknown, SpeakerLayout::Stereo, 100), None);
        assert_eq!(audio_size(AudioFormat::S16, SpeakerLayout::Unknown, 100), None);
        assert_eq!(audio_size(AudioFormat::S16, SpeakerLayout::Mono, 0), None);
    }

    #[test]
    fn differential() {
        let sz = audio_size(AudioFormat::F32, SpeakerLayout::Mono, 480).unwrap();
        assert_equal(sz, 4 * 1 * 480, "audio size");
    }
}
