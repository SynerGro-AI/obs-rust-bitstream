//! Safe pure AddNoise (deterministic noise core).
//!
//! Adds a repeating pattern based on position and amount.
//! Works on Y800. No unwraps.
//!
//! Differential: amount 50 changes a flat frame.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoiseError {
    WrongFormat,
}

pub struct AddNoise;

impl AddNoise {
    /// amount is 0..=255. Uses a simple position hash.
    pub fn apply(frame: &mut VideoFrame, amount: u8) -> Result<(), NoiseError> {
        if frame.format != VideoFormat::Y800 {
            return Err(NoiseError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for (i, p) in plane.iter_mut().enumerate() {
                let n = ((i as u32).wrapping_mul(1103515245).wrapping_add(12345) >> 16) as u8;
                let delta = (n as i16 - 128) * amount as i16 / 255;
                let v = (*p as i16 + delta).clamp(0, 255);
                *p = v as u8;
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
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(128);
        AddNoise::apply(&mut f, 50).unwrap();
        assert!(f.planes[0].iter().any(|p| *p != 128));
    }
}
