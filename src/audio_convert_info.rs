//! Safe pure AudioConvertInfo (OBS audio_convert_info).
//!
//! Parsed from libobs/media-io/audio-io.h.
//! Holds sample rate, format, speaker layout, and clipping.
//! Zero rate is rejected. Channel count matches OBS. No unwraps.
//!
//! Differential: stereo has more channels than mono.

/// Matching OBS audio_format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    Unknown,
    U8,
    S16,
    S32,
    Float,
    U8Planar,
    S16Planar,
    S32Planar,
    FloatPlanar,
}

/// Matching OBS speaker_layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeakerLayout {
    Unknown,
    Mono,
    Stereo,
    TwoPointOne,
    FourPointZero,
    FourPointOne,
    FivePointOne,
    SevenPointOne,
}

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

    /// Channel count matching OBS get_audio_channels.
    pub fn channels(&self) -> u32 {
        match self.speakers {
            SpeakerLayout::Mono => 1,
            SpeakerLayout::Stereo => 2,
            SpeakerLayout::TwoPointOne => 3,
            SpeakerLayout::FourPointZero => 4,
            SpeakerLayout::FourPointOne => 5,
            SpeakerLayout::FivePointOne => 6,
            SpeakerLayout::SevenPointOne => 8,
            SpeakerLayout::Unknown => 2,
        }
    }

    /// Bytes per channel matching OBS get_audio_bytes_per_channel.
    pub fn bytes_per_channel(&self) -> usize {
        match self.format {
            AudioFormat::U8 | AudioFormat::U8Planar => 1,
            AudioFormat::S16 | AudioFormat::S16Planar => 2,
            AudioFormat::S32 | AudioFormat::S32Planar => 4,
            AudioFormat::Float | AudioFormat::FloatPlanar => 4,
            AudioFormat::Unknown => 0,
        }
    }

    /// True if the format is planar.
    pub fn is_planar(&self) -> bool {
        matches!(
            self.format,
            AudioFormat::U8Planar
                | AudioFormat::S16Planar
                | AudioFormat::S32Planar
                | AudioFormat::FloatPlanar
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = AudioConvertInfo::new(48000, AudioFormat::Float, SpeakerLayout::Stereo).unwrap();
        assert_eq!(c.rate(), 48000);
        assert_eq!(c.channels(), 2);
        assert_eq!(c.bytes_per_channel(), 4);
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
