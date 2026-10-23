//! Safe pure AudioOutputInfo (OBS audio_output_info).
//!
//! Parsed from libobs/media-io/audio-io.h.
//! Holds name, sample rate, format, and speaker layout.
//! Empty name or zero rate is rejected. Uses the existing
//! AudioFormat and SpeakerLayout. No unwraps.
//!
//! Differential: a higher rate produces a larger rate.

use crate::audio_format::{AudioFormat, SpeakerLayout};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioInfoError {
    EmptyName,
    ZeroRate,
}

#[derive(Clone, Debug)]
pub struct AudioOutputInfo {
    name: String,
    rate: u32,
    format: AudioFormat,
    speakers: SpeakerLayout,
}

impl AudioOutputInfo {
    /// Create an audio output info. Empty name or zero rate is rejected.
    pub fn new(
        name: &str,
        rate: u32,
        format: AudioFormat,
        speakers: SpeakerLayout,
    ) -> Result<Self, AudioInfoError> {
        if name.is_empty() {
            return Err(AudioInfoError::EmptyName);
        }
        if rate == 0 {
            return Err(AudioInfoError::ZeroRate);
        }
        Ok(Self {
            name: name.to_string(),
            rate,
            format,
            speakers,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
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

    /// Channel count from the speaker layout.
    pub fn channels(&self) -> u32 {
        self.speakers.channels()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let a =
            AudioOutputInfo::new("main", 48000, AudioFormat::F32, SpeakerLayout::Stereo).unwrap();
        assert_eq!(a.name(), "main");
        assert_eq!(a.rate(), 48000);
        assert_eq!(a.channels(), 2);
    }

    #[test]
    fn rejects() {
        assert!(AudioOutputInfo::new("", 48000, AudioFormat::S16, SpeakerLayout::Mono).is_err());
        assert!(AudioOutputInfo::new("main", 0, AudioFormat::S16, SpeakerLayout::Mono).is_err());
    }

    #[test]
    fn differential() {
        let a = AudioOutputInfo::new("a", 44100, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let b = AudioOutputInfo::new("b", 48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        assert!(b.rate() > a.rate());
        assert!(b.channels() > a.channels());
    }
}
