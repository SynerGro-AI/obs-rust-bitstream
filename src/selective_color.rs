//! Safe pure SelectiveColor (range shift core).
//!
//! Shifts only pixels inside a chosen range. Others
//! are left alone. Works on Y800. No unwraps.
//!
//! Differential: pixels in range move, others stay.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectiveError {
    WrongFormat,
    BadRange,
}

pub struct SelectiveColor;

impl SelectiveColor {
    /// low/high define the range. shift is signed.
    pub fn apply(
        frame: &mut VideoFrame,
        low: u8,
        high: u8,
        shift: i8,
    ) -> Result<(), SelectiveError> {
        if frame.format != VideoFormat::Y800 {
            return Err(SelectiveError::WrongFormat);
        }
        if high < low {
            return Err(SelectiveError::BadRange);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                if *p >= low && *p <= high {
                    let v = (*p as i16 + shift as i16).clamp(0, 255);
                    *p = v as u8;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_range() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        assert!(SelectiveColor::apply(&mut f, 200, 100, 10).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![50, 128, 200, 128];
        SelectiveColor::apply(&mut f, 100, 150, 20).unwrap();
        assert_eq!(f.planes[0][0], 50);
        assert_eq!(f.planes[0][1], 148);
        assert_eq!(f.planes[0][2], 200);
    }
}
