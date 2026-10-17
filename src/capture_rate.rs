//! Safe pure CaptureRate (frames per second from a source).
//!
//! Captures a few frames and returns the rate.
//! No unwraps.
//!
//! Differential: the rate is non-negative.

use crate::capture_select::CaptureSource;
use crate::write_timing::WriteTiming;

pub struct CaptureRate;

impl CaptureRate {
    /// Capture n frames and return fps.
    pub fn measure(src: &mut CaptureSource, n: usize) -> f32 {
        if n == 0 {
            return 0.0;
        }
        let timing = WriteTiming::start();
        let mut got = 0;
        for _ in 0..n {
            if src.frame().is_some() {
                got += 1;
            }
        }
        timing.fps(got)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut src = CaptureSource::synthetic(8, 8);
        let rate = CaptureRate::measure(&mut src, 4);
        assert!(rate >= 0.0);
    }
}
