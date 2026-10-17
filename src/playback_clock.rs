//! Safe pure PlaybackClock (frame from elapsed time).
//!
//! Given an fps and elapsed milliseconds, returns
//! the frame index. No unwraps.
//!
//! Differential: 1000 ms at 30 fps is frame 30.

use crate::live_weights::LiveWeights;
use crate::timed_sequence::TimedSequence;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClockError {
    Zero,
}

#[derive(Clone, Debug)]
pub struct PlaybackClock {
    fps: f32,
}

impl PlaybackClock {
    pub fn new(fps: f32) -> Result<Self, ClockError> {
        if fps <= 0.0 {
            return Err(ClockError::Zero);
        }
        Ok(Self { fps })
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Frame index for elapsed milliseconds.
    pub fn frame_at(&self, elapsed_ms: f32) -> usize {
        if elapsed_ms <= 0.0 {
            return 0;
        }
        let secs = elapsed_ms / 1000.0;
        (secs * self.fps).floor() as usize
    }

    /// Elapsed milliseconds for a frame index.
    pub fn time_at(&self, frame: usize) -> f32 {
        frame as f32 / self.fps * 1000.0
    }

    /// Read fps from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(f) = weights.get("fps") {
            if f > 0.0 {
                self.fps = f;
            }
        }
    }

    /// Frame at elapsed time from a timed sequence.
    pub fn frame_from<'a>(
        &self,
        seq: &'a TimedSequence,
        elapsed_ms: f32,
    ) -> Option<&'a VideoFrame> {
        let idx = self.frame_at(elapsed_ms);
        if idx >= seq.len() {
            return None;
        }
        seq.frames().get(idx).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert!(PlaybackClock::new(0.0).is_err());
    }

    #[test]
    fn differential() {
        let mut c = PlaybackClock::new(30.0).unwrap();
        assert_eq!(c.frame_at(0.0), 0);
        assert_eq!(c.frame_at(1000.0), 30);
        assert!((c.time_at(30) - 1000.0).abs() < 1.0);
        let mut w = LiveWeights::new();
        w.set("fps", 60.0).unwrap();
        c.from_weights(&w);
        assert_eq!(c.fps(), 60.0);
        let mut seq = TimedSequence::new(30.0).unwrap();
        for i in 0..10 {
            let mut f = VideoFrame::new(crate::video_fourcc::VideoFormat::Y800, 2, 2).unwrap();
            f.planes[0].fill(i);
            seq.push(f);
        }
        let c2 = PlaybackClock::new(30.0).unwrap();
        let frame = c2.frame_from(&seq, 100.0).unwrap();
        assert_eq!(frame.planes[0][0], 3);
    }
}
