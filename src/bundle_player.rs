//! Safe pure BundlePlayer (step through a bundle core).
//!
//! Uses a PlaybackClock to pick the current audio
//! sample from an AvBundle. No unwraps.
//!
//! Differential: 100 ms at 30 fps is sample 3.

use crate::av_bundle::AvBundle;
use crate::playback_clock::{ClockError, PlaybackClock};

#[derive(Clone, Debug)]
pub struct BundlePlayer {
    clock: PlaybackClock,
}

impl BundlePlayer {
    pub fn new(fps: f32) -> Result<Self, ClockError> {
        Ok(Self {
            clock: PlaybackClock::new(fps)?,
        })
    }

    /// Audio sample at elapsed milliseconds.
    pub fn sample_at(&self, bundle: &AvBundle, elapsed_ms: f32) -> Option<f32> {
        let idx = self.clock.frame_at(elapsed_ms);
        bundle.audio().get(idx).copied()
    }

    /// Fraction through the bundle.
    pub fn fraction(&self, bundle: &AvBundle, elapsed_ms: f32) -> f32 {
        if bundle.audio_len() == 0 {
            return 1.0;
        }
        let idx = self.clock.frame_at(elapsed_ms);
        (idx as f32 / bundle.audio_len() as f32).min(1.0)
    }

    pub fn is_done(&self, bundle: &AvBundle, elapsed_ms: f32) -> bool {
        self.clock.frame_at(elapsed_ms) >= bundle.audio_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut b = AvBundle::new();
        b.push_audio(&[10.0, 20.0, 30.0, 40.0]);
        b.set_fps(30.0);
        let p = BundlePlayer::new(30.0).unwrap();
        assert_eq!(p.sample_at(&b, 0.0), Some(10.0));
        assert_eq!(p.sample_at(&b, 100.0), Some(40.0));
        assert!(p.is_done(&b, 200.0));
    }
}
