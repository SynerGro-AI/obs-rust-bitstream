//! Safe pure AudioMatch (samples → convert info match).
//!
//! Checks whether an audio rate matches an AudioConvertInfo.
//! Returns the rate delta. No unwraps.
//!
//! Differential: a larger target rate produces a larger delta.

use crate::audio_convert_info::AudioConvertInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioMatch {
    pub matches: bool,
    pub src_rate: u32,
    pub dst_rate: u32,
}

impl AudioMatch {
    /// True if the source rate matches the convert info.
    pub fn from(src_rate: u32, info: &AudioConvertInfo) -> Self {
        Self {
            matches: src_rate == info.rate(),
            src_rate,
            dst_rate: info.rate(),
        }
    }

    /// Absolute rate difference.
    pub fn delta(&self) -> u32 {
        self.src_rate.abs_diff(self.dst_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_format::{AudioFormat, SpeakerLayout};

    #[test]
    fn match_same() {
        let info = AudioConvertInfo::new(48000, AudioFormat::F32, SpeakerLayout::Stereo).unwrap();
        let m = AudioMatch::from(48000, &info);
        assert!(m.matches);
        assert_eq!(m.delta(), 0);
    }

    #[test]
    fn differ() {
        let info = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let m = AudioMatch::from(44100, &info);
        assert!(!m.matches);
        assert_eq!(m.delta(), 3900);
    }

    #[test]
    fn differential() {
        let a = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let b = AudioConvertInfo::new(96000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let ma = AudioMatch::from(44100, &a);
        let mb = AudioMatch::from(44100, &b);
        assert!(mb.delta() > ma.delta());
    }
}
