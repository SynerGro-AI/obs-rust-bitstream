//! Safe pure FrameRate (fps control core).
//!
//! Holds a frame rate and computes frame duration.
//! Zero rejected. No unwraps.
//!
//! Differential: 30 fps is about 33 ms per frame.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RateError {
    Zero,
}

#[derive(Clone, Debug)]
pub struct FrameRate {
    fps: f32,
}

impl FrameRate {
    pub fn new(fps: f32) -> Result<Self, RateError> {
        if fps <= 0.0 {
            return Err(RateError::Zero);
        }
        Ok(Self { fps })
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn set(&mut self, fps: f32) -> Result<(), RateError> {
        if fps <= 0.0 {
            return Err(RateError::Zero);
        }
        self.fps = fps;
        Ok(())
    }

    /// Duration of one frame in milliseconds.
    pub fn frame_ms(&self) -> f32 {
        1000.0 / self.fps
    }

    /// Duration of `count` frames in milliseconds.
    pub fn duration_ms(&self, count: usize) -> f32 {
        self.frame_ms() * count as f32
    }

    /// Read fps from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(f) = weights.get("fps") {
            if f > 0.0 {
                self.fps = f;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert!(FrameRate::new(0.0).is_err());
    }

    #[test]
    fn differential() {
        let mut r = FrameRate::new(30.0).unwrap();
        assert!((r.frame_ms() - 33.33).abs() < 0.1);
        assert!((r.duration_ms(30) - 1000.0).abs() < 1.0);
        let mut w = LiveWeights::new();
        w.set("fps", 60.0).unwrap();
        r.from_weights(&w);
        assert_eq!(r.fps(), 60.0);
    }
}
