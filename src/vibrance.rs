//! Safe pure Vibrance (selective saturation core).
//!
//! Boosts mid-range values more than extremes.
//! Works on Y800. No unwraps.
//!
//! Differential: a mid gray is lifted more than a near-black.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VibranceError {
    WrongFormat,
}

pub struct Vibrance;

impl Vibrance {
    /// amount is -100..=100.
    pub fn apply(frame: &mut VideoFrame, amount: i8) -> Result<(), VibranceError> {
        if frame.format != VideoFormat::Y800 {
            return Err(VibranceError::WrongFormat);
        }
        let a = amount as i16;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = *p as i16;
                let dist = (v - 128).abs();
                let weight = 128 - dist.min(128);
                let delta = (a * weight) / 128;
                let out = (v + delta).clamp(0, 255);
                *p = out as u8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![128, 20, 128, 20];
        Vibrance::apply(&mut f, 40).unwrap();
        assert!(f.planes[0][0] > 128);
        assert!(f.planes[0][0] - 128 > f.planes[0][1].saturating_sub(20));
    }
}
