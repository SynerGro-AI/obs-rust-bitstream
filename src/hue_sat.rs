//! Safe pure HueSat (hue/saturation shift core).
//!
//! Shifts a single-channel luminance as a stand-in for
//! hue/sat on Y800. No unwraps.
//!
//! Differential: a mid gray is lifted by a positive shift.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HueSatError {
    WrongFormat,
}

pub struct HueSat;

impl HueSat {
    /// shift is -100..=100. Positive lifts, negative darkens.
    pub fn apply(frame: &mut VideoFrame, shift: i8) -> Result<(), HueSatError> {
        if frame.format != VideoFormat::Y800 {
            return Err(HueSatError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = (*p as i16 + shift as i16).clamp(0, 255);
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
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(128);
        HueSat::apply(&mut f, 20).unwrap();
        assert_eq!(f.planes[0][0], 148);
    }
}
