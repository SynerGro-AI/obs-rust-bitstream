//! Safe pure SequenceTiming (duration from fps core).
//!
//! Computes how long a sequence plays at a given fps.
//! No unwraps.
//!
//! Differential: 30 frames at 30 fps is 1 second.

use crate::fps_control::{FrameRate, RateError};
use crate::frame_sequence::FrameSequence;

pub struct SequenceTiming;

impl SequenceTiming {
    /// Duration of the sequence in milliseconds.
    pub fn duration_ms(seq: &FrameSequence, fps: f32) -> Result<f32, RateError> {
        let rate = FrameRate::new(fps)?;
        Ok(rate.duration_ms(seq.len()))
    }

    /// Duration in seconds.
    pub fn duration_s(seq: &FrameSequence, fps: f32) -> Result<f32, RateError> {
        Ok(Self::duration_ms(seq, fps)? / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    #[test]
    fn zero() {
        let seq = FrameSequence::new();
        assert!(SequenceTiming::duration_ms(&seq, 0.0).is_err());
    }

    #[test]
    fn differential() {
        let mut seq = FrameSequence::new();
        for _ in 0..30 {
            let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
            seq.push(f);
        }
        let ms = SequenceTiming::duration_ms(&seq, 30.0).unwrap();
        assert!((ms - 1000.0).abs() < 1.0);
        let s = SequenceTiming::duration_s(&seq, 30.0).unwrap();
        assert!((s - 1.0).abs() < 0.01);
    }
}
