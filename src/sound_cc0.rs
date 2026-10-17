//! Safe pure SoundCc0 (CC0 sample catalog core).
//!
//! Lists the sound-cc0 files and can load a WAV
//! from bytes. No unwraps.
//!
//! Differential: the catalog is non-empty.

use crate::wav_reader::{WavData, WavError, WavReader};

/// CC0 files from https://github.com/code4fukui/sound-cc0
pub const SOUND_CC0: &[&str] = &[
    "bell1.wav",
    "cracker1.wav",
    "cracker1v.wav",
    "cracker1v-stereo.wav",
    "cracker2.wav",
    "metal1.wav",
    "steel1.wav",
    "switch1.wav",
    "wahaha.wav",
];

const BASE: &str = "https://raw.githubusercontent.com/code4fukui/sound-cc0/main/";

pub struct SoundCc0;

impl SoundCc0 {
    /// Number of files in the catalog.
    pub fn len() -> usize {
        SOUND_CC0.len()
    }

    /// File name at index.
    pub fn name(index: usize) -> Option<&'static str> {
        SOUND_CC0.get(index).copied()
    }

    /// Raw URL for a file.
    pub fn url(name: &str) -> String {
        format!("{}{}", BASE, name)
    }

    /// Parse WAV bytes into samples.
    pub fn load(bytes: &[u8]) -> Result<WavData, WavError> {
        WavReader::parse(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        assert!(SoundCc0::len() > 0);
        assert_eq!(SoundCc0::name(0), Some("bell1.wav"));
        assert!(SoundCc0::url("bell1.wav").contains("bell1.wav"));
    }
}
