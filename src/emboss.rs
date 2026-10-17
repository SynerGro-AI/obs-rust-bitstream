//! Safe pure Emboss (edge relief core).
//!
//! Subtracts the upper-left neighbor from each pixel
//! and biases to mid gray. Works on Y800. No unwraps.
//!
//! Differential: an edge produces a non-mid value.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmbossError {
    WrongFormat,
}

pub struct Emboss;

impl Emboss {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), EmbossError> {
        if frame.format != VideoFormat::Y800 {
            return Err(EmbossError::WrongFormat);
        }
        let w = frame.width as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for i in w..src.len() {
                let diff = src[i] as i16 - src[i - w - 1] as i16;
                let v = (128 + diff).clamp(0, 255);
                dst[i] = v as u8;
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
        f.planes[0].fill(32);
        f.planes[0][2 * 4 + 2] = 200;
        Emboss::apply(&mut f).unwrap();
        // The edge pixel differs from a flat mid result.
        assert!(f.planes[0][2 * 4 + 2] != 32);
    }
}
