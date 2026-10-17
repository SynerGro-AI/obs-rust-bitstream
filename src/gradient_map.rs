//! Safe pure GradientMap (luminance to ramp core).
//!
//! Maps each pixel through a two-stop gradient.
//! 0 maps to start, 255 maps to end. Works on Y800.
//! No unwraps.
//!
//! Differential: 0 stays start, 255 becomes end.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GradientMapError {
    WrongFormat,
}

pub struct GradientMap;

impl GradientMap {
    pub fn apply(frame: &mut VideoFrame, start: u8, end: u8) -> Result<(), GradientMapError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GradientMapError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let t = *p as u16;
                let v = start as u16 + ((end as i16 - start as i16) as u16 * t) / 255;
                *p = v.min(255) as u8;
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
        f.planes[0] = vec![0, 128, 255, 64];
        GradientMap::apply(&mut f, 0, 200).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][2], 200);
        assert!(f.planes[0][1] > 0 && f.planes[0][1] < 200);
    }
}
