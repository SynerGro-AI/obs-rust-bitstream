//! Safe pure TimedSequence (frames + fps core).
//!
//! Holds a sequence and an fps. Computes duration.
//! Zero fps rejected. No unwraps.
//!
//! Differential: 30 frames at 30 fps is 1 second.

use crate::fps_control::{FrameRate, RateError};
use crate::frame_sequence::FrameSequence;
use crate::live_weights::LiveWeights;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug)]
pub struct TimedSequence {
    frames: FrameSequence,
    fps: f32,
}

impl TimedSequence {
    pub fn new(fps: f32) -> Result<Self, RateError> {
        if fps <= 0.0 {
            return Err(RateError::Zero);
        }
        Ok(Self {
            frames: FrameSequence::new(),
            fps,
        })
    }

    pub fn push(&mut self, frame: VideoFrame) {
        self.frames.push(frame);
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frames(&self) -> &FrameSequence {
        &self.frames
    }

    /// Duration in milliseconds.
    pub fn duration_ms(&self) -> f32 {
        FrameRate::new(self.fps)
            .map(|r| r.duration_ms(self.frames.len()))
            .unwrap_or(0.0)
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
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn zero() {
        assert!(TimedSequence::new(0.0).is_err());
    }

    #[test]
    fn differential() {
        let mut t = TimedSequence::new(30.0).unwrap();
        for _ in 0..30 {
            let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
            t.push(f);
        }
        assert!((t.duration_ms() - 1000.0).abs() < 1.0);
        let mut w = LiveWeights::new();
        w.set("fps", 60.0).unwrap();
        t.from_weights(&w);
        assert_eq!(t.fps(), 60.0);
    }
}
