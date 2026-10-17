//! Safe pure AudioWeight (live weights into audio gain).
//!
//! Reads a named weight and applies it as gain.
//! Missing weight is skipped. No unwraps.
//!
//! Differential: a weight of 0.5 halves a peak of 1.0.

use crate::audio_mixer;
use crate::live_weights::LiveWeights;

pub struct AudioWeight;

impl AudioWeight {
    /// Apply gain from a named weight. Returns the gained samples.
    pub fn gain(samples: &[f32], weights: &LiveWeights, name: &str) -> Vec<f32> {
        let g = weights.get(name).unwrap_or(1.0);
        audio_mixer::apply_gain(samples, g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let samples = vec![1.0, -1.0, 0.5];
        let mut w = LiveWeights::new();
        w.set("audio.gain", 0.5).unwrap();
        let out = AudioWeight::gain(&samples, &w, "audio.gain");
        assert_eq!(out[0], 0.5);
        assert_eq!(out[1], -0.5);
    }
}
