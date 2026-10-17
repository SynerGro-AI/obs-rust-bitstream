//! Safe pure Invert (pixel invert core).
//!
//! Replaces each pixel with 255 - pixel. Works on Y800.
//! No unwraps.
//!
//! Differential: 64 becomes 191.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvertError {
    WrongFormat,
}

pub struct Invert;

impl Invert {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), InvertError> {
        if frame.format != VideoFormat::Y800 {
            return Err(InvertError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = 255 - *p;
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
        f.planes[0].fill(64);
        Invert::apply(&mut f).unwrap();
        assert_eq!(f.planes[0][0], 191);
    }
}
