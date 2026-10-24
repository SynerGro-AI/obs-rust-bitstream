//! Safe pure FrameFilter (Y800 brightness core).
//!
//! Adjusts every pixel of a Y800 frame by a signed delta.
//! Clamps to 0..=255. No unwraps.
//!
//! Differential: +10 lifts a mid gray, -300 clamps to 0.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterError {
    WrongFormat,
}

pub struct FrameFilter;

impl FrameFilter {
    /// Add `delta` to every pixel. delta is signed.
    pub fn brightness(frame: &mut VideoFrame, delta: i16) -> Result<(), FilterError> {
        if frame.format != VideoFormat::Y800 {
            return Err(FilterError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = *p as i16 + delta;
                *p = v.clamp(0, 255) as u8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_format() {
        let mut f = VideoFrame::new(VideoFormat::Yuy2, 2, 2).unwrap();
        assert!(FrameFilter::brightness(&mut f, 10).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(128);
        FrameFilter::brightness(&mut f, 10).unwrap();
        assert_eq!(f.planes[0][0], 138);
        FrameFilter::brightness(&mut f, -300).unwrap();
        assert_eq!(f.planes[0][0], 0);
    }
}
