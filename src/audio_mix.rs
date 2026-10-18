//! Safe pure AudioMix (pull from AudioSourceList).
//!
//! Pulls samples from the active source, applies a gain,
//! and returns the result. Empty list or zero frames
//! return None. No unwraps.
//!
//! Differential: gain changes the peak.

use crate::audio_mixer::{apply_gain, peak};
use crate::audio_source_list::AudioSourceList;

#[derive(Clone, Debug)]
pub struct AudioMix {
    gain: f32,
}

impl AudioMix {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    /// Pull frames from the active source and apply gain.
    /// Returns None if the list is empty or the source yields nothing.
    pub fn pull(&self, list: &mut AudioSourceList, frames: usize) -> Option<Vec<f32>> {
        if frames == 0 || list.is_empty() {
            return None;
        }
        let samples = list.samples(frames)?;
        Some(apply_gain(&samples.data, self.gain))
    }

    /// Peak of a pull. None if nothing was produced.
    pub fn peak(&self, list: &mut AudioSourceList, frames: usize) -> Option<f32> {
        self.pull(list, frames).map(|v| peak(&v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;

    #[test]
    fn pull_with_gain() {
        let mut list = AudioSourceList::new();
        list.add("tone", AudioSource::synthetic(48000, 1));
        let mix = AudioMix::new(2.0);
        let out = mix.pull(&mut list, 8).unwrap();
        assert_eq!(out.len(), 8);
        assert!(out.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn empty_list() {
        let mut list = AudioSourceList::new();
        let mix = AudioMix::new(1.0);
        assert!(mix.pull(&mut list, 8).is_none());
    }

    #[test]
    fn differential_gain() {
        let mut list = AudioSourceList::new();
        list.add("tone", AudioSource::synthetic(48000, 1));
        let low = AudioMix::new(0.5).peak(&mut list, 16).unwrap();
        // Reset by pulling again; phase advances, so compare shape via gain.
        let high = AudioMix::new(2.0).peak(&mut list, 16).unwrap();
        assert!(high > low);
    }
}
