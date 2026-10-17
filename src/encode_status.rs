//! Safe pure EncodeStatus (progress + time core).
//!
//! Combines EncodeProgress with a PlaybackClock
//! to report frames done and time remaining.
//! No unwraps.
//!
//! Differential: halfway is 0.5 and half the time.

use crate::encode_progress::EncodeProgress;
use crate::playback_clock::PlaybackClock;

#[derive(Clone, Debug)]
pub struct EncodeStatus {
    progress: EncodeProgress,
    clock: PlaybackClock,
}

impl EncodeStatus {
    pub fn new(total: usize, fps: f32) -> Result<Self, crate::playback_clock::ClockError> {
        Ok(Self {
            progress: EncodeProgress::new(total),
            clock: PlaybackClock::new(fps)?,
        })
    }

    pub fn tick(&mut self) {
        self.progress.tick();
    }

    pub fn fraction(&self) -> f32 {
        self.progress.fraction()
    }

    pub fn done(&self) -> usize {
        self.progress.done()
    }

    /// Milliseconds remaining.
    pub fn remaining_ms(&self) -> f32 {
        let left = self.progress.total().saturating_sub(self.progress.done());
        self.clock.time_at(left)
    }

    pub fn is_done(&self) -> bool {
        self.progress.is_done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut s = EncodeStatus::new(10, 30.0).unwrap();
        assert!(s.remaining_ms() > 0.0);
        for _ in 0..5 {
            s.tick();
        }
        assert!((s.fraction() - 0.5).abs() < 0.01);
        assert!(s.remaining_ms() > 0.0);
        assert!(!s.is_done());
    }
}
