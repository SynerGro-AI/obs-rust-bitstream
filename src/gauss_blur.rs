//! Safe pure GaussBlur (multi-pass box blur core).
//!
//! Runs box blur `passes` times. Works on Y800.
//! No unwraps.
//!
//! Differential: more passes reduce a lone bright pixel further.

use crate::box_blur::BoxBlur;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GaussError {
    WrongFormat,
}

pub struct GaussBlur;

impl GaussBlur {
    pub fn apply(frame: &mut VideoFrame, passes: u8) -> Result<(), GaussError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GaussError::WrongFormat);
        }
        for _ in 0..passes.max(1) {
            BoxBlur::apply(frame).map_err(|_| GaussError::WrongFormat)?;
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
        f.planes[0][1 * 4 + 1] = 255;
        GaussBlur::apply(&mut f, 2).unwrap();
        assert!(f.planes[0][1 * 4 + 1] < 200);
    }
}
