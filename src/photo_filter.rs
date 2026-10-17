//! Safe pure PhotoFilter (warm/cool tint core).
//!
//! Shifts luminance toward warm (lift) or cool (darken).
//! Works on Y800. No unwraps.
//!
//! Differential: a warm filter lifts a mid gray.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterTint {
    Warm,
    Cool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhotoFilterError {
    WrongFormat,
}

pub struct PhotoFilter;

impl PhotoFilter {
    /// density is 0..=100.
    pub fn apply(
        frame: &mut VideoFrame,
        tint: FilterTint,
        density: u8,
    ) -> Result<(), PhotoFilterError> {
        if frame.format != VideoFormat::Y800 {
            return Err(PhotoFilterError::WrongFormat);
        }
        let d = density.min(100) as i16;
        let shift = match tint {
            FilterTint::Warm => d / 2,
            FilterTint::Cool => -(d / 2),
        };
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = (*p as i16 + shift).clamp(0, 255);
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
        PhotoFilter::apply(&mut f, FilterTint::Warm, 40).unwrap();
        assert_eq!(f.planes[0][0], 148);
        PhotoFilter::apply(&mut f, FilterTint::Cool, 40).unwrap();
        assert_eq!(f.planes[0][0], 128);
    }
}
