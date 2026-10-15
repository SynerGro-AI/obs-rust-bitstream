//! Safe pure ColorBalance (shadow/mid/highlight shift core).
//!
//! Shifts luminance in three ranges. Works on Y800.
//! No unwraps.
//!
//! Differential: a mid gray is shifted by the mid control.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalanceError {
    WrongFormat,
}

pub struct ColorBalance;

impl ColorBalance {
    /// shadow, mid, highlight are signed shifts, -100..=100.
    pub fn apply(
        frame: &mut VideoFrame,
        shadow: i8,
        mid: i8,
        highlight: i8,
    ) -> Result<(), BalanceError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BalanceError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = *p;
                let shift = if v < 85 {
                    shadow
                } else if v < 170 {
                    mid
                } else {
                    highlight
                };
                let out = (v as i16 + shift as i16).clamp(0, 255);
                *p = out as u8;
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
        ColorBalance::apply(&mut f, 0, 20, 0).unwrap();
        assert_eq!(f.planes[0][0], 148);
    }
}
