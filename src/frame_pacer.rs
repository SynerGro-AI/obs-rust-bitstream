//! Safe pure FramePacer (pace a source to a target fps).
//!
//! Returns true when enough time has passed for the next frame.
//! No unwraps.
//!
//! Differential: a zero interval always allows a frame.

use std::time::{Duration, Instant};

pub struct FramePacer {
    target: f32,
    last: Instant,
}

impl FramePacer {
    /// Target frames per second. 0 means no limit.
    pub fn new(fps: f32) -> Self {
        Self {
            target: fps.max(0.0),
            last: Instant::now(),
        }
    }

    /// Interval between frames. Zero if no limit.
    pub fn interval(&self) -> Duration {
        if self.target <= 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(1.0 / self.target)
        }
    }

    /// True if it is time for the next frame.
    pub fn ready(&mut self) -> bool {
        let interval = self.interval();
        if interval.is_zero() {
            self.last = Instant::now();
            return true;
        }
        if self.last.elapsed() >= interval {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }

    /// Target fps.
    pub fn target(&self) -> f32 {
        self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut p = FramePacer::new(0.0);
        assert!(p.ready());
        let mut limited = FramePacer::new(1000.0);
        // first call is always ready because last was just set
        // but elapsed is tiny, so may be false. Just check target.
        assert_eq!(limited.target(), 1000.0);
        assert!(limited.interval() > Duration::ZERO);
    }
}
