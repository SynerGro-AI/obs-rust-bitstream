//! Safe pure BrightContrast (brightness and contrast core).
//!
//! Brightness adds a signed offset. Contrast scales
//! around mid gray. Works on Y800. No unwraps.
//!
//! Differential: +20 brightness lifts a mid gray.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrightContrastError {
    WrongFormat,
}

pub struct BrightContrast;

impl BrightContrast {
    /// brightness and contrast are -100..=100.
    pub fn apply(
        frame: &mut VideoFrame,
        brightness: i8,
        contrast: i8,
    ) -> Result<(), BrightContrastError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BrightContrastError::WrongFormat);
        }
        let b = brightness as i16;
        let c = contrast as i16;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = *p as i16;
                let shifted = v + b;
                let scaled = 128 + ((shifted - 128) * (100 + c)) / 100;
                *p = scaled.clamp(0, 255) as u8;
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
        BrightContrast::apply(&mut f, 20, 0).unwrap();
        assert_eq!(f.planes[0][0], 148);
    }
}
