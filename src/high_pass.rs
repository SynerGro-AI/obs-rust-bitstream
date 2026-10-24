//! Safe pure HighPass (detail extract core).
//!
//! Subtracts a blurred copy from the original and
//! biases to mid gray. Works on Y800. No unwraps.
//!
//! Differential: a flat area stays near 128, an edge moves.

use crate::box_blur::BoxBlur;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighPassError {
    WrongFormat,
}

pub struct HighPass;

impl HighPass {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), HighPassError> {
        if frame.format != VideoFormat::Y800 {
            return Err(HighPassError::WrongFormat);
        }
        let mut blurred = frame.clone();
        BoxBlur::apply(&mut blurred).map_err(|_| HighPassError::WrongFormat)?;
        if let (Some(dst), Some(blur)) = (frame.planes.get_mut(0), blurred.planes.first()) {
            for (d, b) in dst.iter_mut().zip(blur.iter()) {
                let v = 128 + (*d as i16 - *b as i16);
                *d = v.clamp(0, 255) as u8;
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
        f.planes[0].fill(64);
        f.planes[0][4 + 1] = 200;
        HighPass::apply(&mut f).unwrap();
        assert!(f.planes[0][4 + 1] > 128);
    }
}
