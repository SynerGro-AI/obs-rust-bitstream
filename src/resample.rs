//! Safe pure Resample (change fps core).
//!
//! Picks frames to match a target fps. No unwraps.
//!
//! Differential: 30 frames at 30 fps resampled to
//! 15 fps yields about 15 frames.

use crate::fps_control::RateError;
use crate::timed_sequence::TimedSequence;

pub struct Resample;

impl Resample {
    /// Resample a timed sequence to a new fps.
    pub fn to_fps(src: &TimedSequence, new_fps: f32) -> Result<TimedSequence, RateError> {
        if new_fps <= 0.0 {
            return Err(RateError::Zero);
        }
        let mut out = TimedSequence::new(new_fps)?;
        if src.is_empty() {
            return Ok(out);
        }
        let ratio = new_fps / src.fps();
        let new_count = ((src.len() as f32) * ratio).round().max(1.0) as usize;
        for i in 0..new_count {
            let src_i = ((i as f32) / ratio).floor() as usize;
            let src_i = src_i.min(src.len() - 1);
            if let Ok(f) = src.frames().get(src_i) {
                out.push(f.clone());
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    #[test]
    fn zero() {
        let t = TimedSequence::new(30.0).unwrap();
        assert!(Resample::to_fps(&t, 0.0).is_err());
    }

    #[test]
    fn differential() {
        let mut src = TimedSequence::new(30.0).unwrap();
        for i in 0..30 {
            let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
            f.planes[0].fill(i as u8);
            src.push(f);
        }
        let out = Resample::to_fps(&src, 15.0).unwrap();
        assert_eq!(out.fps(), 15.0);
        assert!(out.len() >= 14 && out.len() <= 16);
    }
}
