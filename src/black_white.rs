//! Safe pure BlackWhite (desaturate core).
//!
//! On Y800 the frame is already luminance. This clamps
//! to a pure black/white split at mid gray so the result
//! is strictly two tones. No unwraps.
//!
//! Differential: 100 becomes 0, 200 becomes 255.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlackWhiteError {
    WrongFormat,
}

pub struct BlackWhite;

impl BlackWhite {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), BlackWhiteError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BlackWhiteError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = if *p >= 128 { 255 } else { 0 };
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
        f.planes[0] = vec![100, 200, 127, 128];
        BlackWhite::apply(&mut f).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][1], 255);
        assert_eq!(f.planes[0][2], 0);
        assert_eq!(f.planes[0][3], 255);
    }
}
