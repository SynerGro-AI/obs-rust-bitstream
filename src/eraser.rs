//! Safe pure Eraser (clear stamp core).
//!
//! Clears a square to 0 at (x, y). Out-of-bounds rejected.
//! Works on Y800. No unwraps.
//!
//! Differential: a filled pixel is cleared.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EraserError {
    WrongFormat,
    OutOfBounds,
}

pub struct Eraser;

impl Eraser {
    pub fn clear(frame: &mut VideoFrame, x: u32, y: u32, size: u8) -> Result<(), EraserError> {
        if frame.format != VideoFormat::Y800 {
            return Err(EraserError::WrongFormat);
        }
        let s = size.max(1) as u32;
        if x.saturating_add(s) > frame.width || y.saturating_add(s) > frame.height {
            return Err(EraserError::OutOfBounds);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for row in y..y + s {
                for col in x..x + s {
                    let i = (row * frame.width + col) as usize;
                    if i < plane.len() {
                        plane[i] = 0;
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
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(200);
        Eraser::clear(&mut f, 1, 1, 2).unwrap();
        assert_eq!(f.planes[0][1 * 4 + 1], 0);
        assert_eq!(f.planes[0][0], 200);
    }
}
