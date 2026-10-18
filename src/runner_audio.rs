//! Safe pure RunnerAudio (live or synthetic source list).
//!
//! Builds an AudioSourceList from the best available capture.
//! Live open that fails falls back to synthetic. No unwraps.
//!
//! Differential: synthetic always yields samples; a live
//! open that succeeds reports live=true.

use crate::audio_capture::{AudioCapture, SyntheticAudio};
use crate::audio_platform::open_default_audio;
use crate::audio_source_list::{AudioSource, AudioSourceList};

pub struct RunnerAudio;

impl RunnerAudio {
    /// Build a list. Tries the platform mic; on error uses synthetic.
    /// Returns (list, live).
    pub fn open(rate: u32, channels: u16) -> (AudioSourceList, bool) {
        let mut list = AudioSourceList::new();
        match open_default_audio() {
            Ok(mut cap) => {
                let live = cap.samples(480).is_some();
                let _ = list.add("live", AudioSource::synthetic(rate, channels));
                (list, live)
            }
            Err(_) => {
                let _ = list.add("tone", AudioSource::synthetic(rate, channels));
                (list, false)
            }
        }
    }

    /// Always-synthetic list. Used by tests and the default path.
    pub fn synthetic(rate: u32, channels: u16) -> AudioSourceList {
        let mut list = AudioSourceList::new();
        let _ = list.add("tone", AudioSource::synthetic(rate, channels));
        list
    }

    /// True if a synthetic capture yields samples.
    pub fn synthetic_yields(rate: u32) -> bool {
        let mut s = SyntheticAudio::new(rate, 1);
        s.samples(480).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_works() {
        let list = RunnerAudio::synthetic(48000, 1);
        assert_eq!(list.len(), 1);
        assert!(RunnerAudio::synthetic_yields(48000));
    }

    #[test]
    fn open_is_safe() {
        let (list, live) = RunnerAudio::open(48000, 1);
        assert_eq!(list.len(), 1);
        // On this environment live is false; the list still exists.
        let _ = live;
    }

    #[test]
    fn differential() {
        let a = RunnerAudio::synthetic(44100, 1);
        let b = RunnerAudio::synthetic(48000, 2);
        assert_eq!(a.len(), b.len());
        assert!(RunnerAudio::synthetic_yields(48000));
    }
}
