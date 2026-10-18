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

    /// True if the measured rate is within a fraction of the target.
    /// A target of 0 always hits.
    pub fn hit(target: f32, measured: f32, fraction: f32) -> bool {
        if target <= 0.0 {
            return true;
        }
        let delta = (measured - target).abs();
        delta <= target * fraction
    }

    /// Measured minus target. Positive means faster.
    pub fn delta(target: f32, measured: f32) -> f32 {
        measured - target
    }

    /// Collect up to n frames, waiting for the pacer between each.
    /// Stops early if the source returns None.
    pub fn collect(
        &mut self,
        src: &mut crate::capture_select::CaptureSource,
        n: usize,
    ) -> Vec<crate::video_frame::VideoFrame> {
        let mut out = Vec::new();
        let mut attempts = 0;
        let max_attempts = n.saturating_mul(10).max(1);
        while out.len() < n && attempts < max_attempts {
            attempts += 1;
            if self.ready() {
                if let Some(f) = src.frame() {
                    out.push(f);
                } else {
                    break;
                }
            } else {
                // spin briefly so we don't burn a core
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        out
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
        let mut src = crate::capture_select::CaptureSource::synthetic(4, 4);
        let mut paced = FramePacer::new(0.0); // no limit
        let frames = paced.collect(&mut src, 3);
        assert_eq!(frames.len(), 3);
        assert!(FramePacer::hit(30.0, 31.0, 0.1));
        assert!(!FramePacer::hit(30.0, 10.0, 0.1));
        assert!((FramePacer::delta(30.0, 40.0) - 10.0).abs() < 0.1);
    }
}
