//! Safe pure Posterize (level reduce core).
//!
//! Reduces the frame to a fixed number of levels.
//! Works on Y800. No unwraps.
//!
//! Differential: 4 levels map 100 and 200 to different steps.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PosterizeError {
    WrongFormat,
    BadLevels,
}

pub struct Posterize;

impl Posterize {
    /// levels is 2..=255.
    pub fn apply(frame: &mut VideoFrame, levels: u8) -> Result<(), PosterizeError> {
        if frame.format != VideoFormat::Y800 {
            return Err(PosterizeError::WrongFormat);
        }
        if levels < 2 {
            return Err(PosterizeError::BadLevels);
        }
        let step = 255u16 / (levels as u16 - 1);
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let band = (*p as u16 * (levels as u16 - 1)) / 255;
                *p = (band * step).min(255) as u8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_levels() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        assert!(Posterize::apply(&mut f, 1).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![0, 100, 200, 255];
        Posterize::apply(&mut f, 4).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][3], 255);
        assert!(f.planes[0][1] < f.planes[0][2]);
    }
}
