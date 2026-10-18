//! Safe pure AudioConvertInfo (OBS audio_convert_info).
//!
//! Parsed from libobs/media-io/audio-io.h.
//! Holds sample rate, format, speaker layout, and clipping.
//! Zero rate is rejected. Uses the existing AudioFormat and
//! SpeakerLayout. No unwraps.
//!
//! Differential: stereo has more channels than mono.

use crate::audio_format::{AudioFormat, SpeakerLayout};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConvertError {
    ZeroRate,
}

#[derive(Clone, Debug)]
pub struct AudioConvertInfo {
    rate: u32,
    format: AudioFormat,
    speakers: SpeakerLayout,
    allow_clipping: bool,
}

impl AudioConvertInfo {
    /// Create a convert request. Zero rate is rejected.
    pub fn new(rate: u32, format: AudioFormat, speakers: SpeakerLayout) -> Result<Self, ConvertError> {
        if rate == 0 {
            return Err(ConvertError::ZeroRate);
        }
        Ok(Self {
            rate,
            format,
            speakers,
            allow_clipping: false,
        })
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn speakers(&self) -> SpeakerLayout {
        self.speakers
    }

    pub fn allow_clipping(&self) -> bool {
        self.allow_clipping
    }

    pub fn set_allow_clipping(&mut self, allow: bool) {
        self.allow_clipping = allow;
    }

    /// Channel count from the existing SpeakerLayout.
    pub fn channels(&self) -> u32 {
        self.speakers.channels()
    }

    /// True if the format is planar.
    pub fn is_planar(&self) -> bool {
        matches!(
            self.format,
            AudioFormat::U8Planar
                | AudioFormat::S16Planar
                | AudioFormat::S32Planar
                | AudioFormat::F32Planar
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = AudioConvertInfo::new(48000, AudioFormat::F32, SpeakerLayout::Stereo).unwrap();
        assert_eq!(c.rate(), 48000);
        assert_eq!(c.channels(), 2);
        assert!(!c.is_planar());
    }

    #[test]
    fn zero_rejected() {
        assert!(AudioConvertInfo::new(0, AudioFormat::S16, SpeakerLayout::Mono).is_err());
    }

    #[test]
    fn planar() {
        let c = AudioConvertInfo::new(44100, AudioFormat::S16Planar, SpeakerLayout::Mono).unwrap();
        assert!(c.is_planar());
        assert_eq!(c.channels(), 1);
    }

    #[test]
    fn differential() {
        let a = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let b = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        assert!(b.channels() > a.channels());
    }
}