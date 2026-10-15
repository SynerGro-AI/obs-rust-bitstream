//! Safe pure Levels (input/output range core).
//!
//! Remaps luminance from an input range to an output range.
//! Works on Y800. No unwraps.
//!
//! Differential: a mid gray is lifted when input max is lowered.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelsError {
    WrongFormat,
    BadRange,
}

pub struct Levels;

impl Levels {
    /// in_min/in_max and out_min/out_max are 0..=255.
    pub fn apply(
        frame: &mut VideoFrame,
        in_min: u8,
        in_max: u8,
        out_min: u8,
        out_max: u8,
    ) -> Result<(), LevelsError> {
        if frame.format != VideoFormat::Y800 {
            return Err(LevelsError::WrongFormat);
        }
        if in_max <= in_min || out_max < out_min {
            return Err(LevelsError::BadRange);
        }
        let in_span = (in_max - in_min) as u32;
        let out_span = (out_max - out_min) as u32;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = (*p).clamp(in_min, in_max);
                let t = (v - in_min) as u32;
                let mapped = out_min as u32 + (t * out_span) / in_span;
                *p = mapped.min(255) as u8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_range() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        assert!(Levels::apply(&mut f, 200, 100, 0, 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(128);
        Levels::apply(&mut f, 0, 128, 0, 255).unwrap();
        assert_eq!(f.planes[0][0], 255);
    }
}
