//! Safe pure CloneStamp (copy-from-offset core).
//!
//! Copies a square from a source offset into the
//! destination. Out-of-bounds rejected. Works on Y800.
//! No unwraps.
//!
//! Differential: a source pixel appears at the destination.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloneError {
    WrongFormat,
    OutOfBounds,
}

pub struct CloneStamp;

impl CloneStamp {
    /// Copy a size x size square from (sx, sy) to (dx, dy).
    pub fn stamp(
        frame: &mut VideoFrame,
        sx: u32,
        sy: u32,
        dx: u32,
        dy: u32,
        size: u8,
    ) -> Result<(), CloneError> {
        if frame.format != VideoFormat::Y800 {
            return Err(CloneError::WrongFormat);
        }
        let s = size.max(1) as u32;
        if sx.saturating_add(s) > frame.width
            || sy.saturating_add(s) > frame.height
            || dx.saturating_add(s) > frame.width
            || dy.saturating_add(s) > frame.height
        {
            return Err(CloneError::OutOfBounds);
        }
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for row in 0..s {
                for col in 0..s {
                    let si = ((sy + row) * frame.width + (sx + col)) as usize;
                    let di = ((dy + row) * frame.width + (dx + col)) as usize;
                    if si < src.len() && di < dst.len() {
                        dst[di] = src[si];
                    }
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
    fn out_of_bounds() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        assert!(CloneStamp::stamp(&mut f, 3, 3, 0, 0, 2).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0][0] = 200;
        CloneStamp::stamp(&mut f, 0, 0, 2, 2, 1).unwrap();
        assert_eq!(f.planes[0][2 * 4 + 2], 200);
    }
}
