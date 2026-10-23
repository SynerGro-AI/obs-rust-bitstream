//! Safe pure Exposure (stops shift core).
//!
//! Multiplies luminance by 2^stops. Works on Y800.
//! No unwraps.
//!
//! Differential: +1 stop doubles a mid gray.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExposureError {
    WrongFormat,
}

pub struct Exposure;

impl Exposure {
    /// stops is a signed integer. +1 doubles, -1 halves.
    pub fn apply(frame: &mut VideoFrame, stops: i8) -> Result<(), ExposureError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ExposureError::WrongFormat);
        }
        let factor = if stops >= 0 {
            1u32 << (stops as u32)
        } else {
            1
        };
        let div = if stops < 0 {
            1u32 << ((-stops) as u32)
        } else {
            1
        };
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = (*p as u32 * factor) / div;
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
        f.planes[0].fill(64);
        Exposure::apply(&mut f, 1).unwrap();
        assert_eq!(f.planes[0][0], 128);
    }
}
