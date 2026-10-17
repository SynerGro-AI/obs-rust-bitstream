//! Safe pure WriteTiming (elapsed time core).
//!
//! Records the start and returns elapsed
//! milliseconds. No unwraps.
//!
//! Differential: elapsed is non-negative.

use std::time::Instant;

#[derive(Clone, Debug)]
pub struct WriteTiming {
    start: Instant,
}

impl WriteTiming {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Elapsed milliseconds.
    pub fn elapsed_ms(&self) -> f32 {
        self.start.elapsed().as_secs_f32() * 1000.0
    }

    /// Frames per second from a count.
    pub fn fps(&self, frames: usize) -> f32 {
        let secs = self.elapsed_ms() / 1000.0;
        if secs <= 0.0 {
            return 0.0;
        }
        frames as f32 / secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let t = WriteTiming::start();
        assert!(t.elapsed_ms() >= 0.0);
        assert!(t.fps(10) >= 0.0);
    }
}
