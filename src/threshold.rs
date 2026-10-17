//! Safe pure Threshold (binary split core).
//!
//! Pixels above the threshold become 255, others 0.
//! Works on Y800. No unwraps.
//!
//! Differential: 100 with threshold 128 becomes 0.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThresholdError {
    WrongFormat,
}

pub struct Threshold;

impl Threshold {
    pub fn apply(frame: &mut VideoFrame, level: u8) -> Result<(), ThresholdError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ThresholdError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = if *p >= level { 255 } else { 0 };
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
        f.planes[0] = vec![100, 200, 50, 128];
        Threshold::apply(&mut f, 128).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][1], 255);
        assert_eq!(f.planes[0][3], 255);
    }
}
