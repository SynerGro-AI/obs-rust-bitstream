//! Safe pure Brush (stamp core).
//!
//! Paints a square of the given size and color at (x, y).
//! Out-of-bounds rejected. Works on Y800. No unwraps.
//!
//! Differential: a 2x2 stamp sets those pixels.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrushError {
    WrongFormat,
    OutOfBounds,
}

pub struct Brush;

impl Brush {
    pub fn stamp(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        size: u8,
        color: u8,
    ) -> Result<(), BrushError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BrushError::WrongFormat);
        }
        let s = size.max(1) as u32;
        if x.saturating_add(s) > frame.width || y.saturating_add(s) > frame.height {
            return Err(BrushError::OutOfBounds);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for row in y..y + s {
                for col in x..x + s {
                    let i = (row * frame.width + col) as usize;
                    if i < plane.len() {
                        plane[i] = color;
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
        assert!(Brush::stamp(&mut f, 3, 3, 2, 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        Brush::stamp(&mut f, 1, 1, 2, 200).unwrap();
        assert_eq!(f.planes[0][1 * 4 + 1], 200);
        assert_eq!(f.planes[0][0], 0);
    }
}
