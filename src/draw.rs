//! Safe pure Draw (rectangle into Y800 frame).
//!
//! Fills a rectangle with a luminance. Out-of-bounds
//! rejected. No unwraps.
//!
//! Differential: a 2x2 rect at (1,1) sets those four
//! pixels and leaves the rest.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawError {
    WrongFormat,
    OutOfBounds,
}

pub struct Draw;

impl Draw {
    /// Fill a rectangle. x, y, w, h are in pixels.
    pub fn rect(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        lum: u8,
    ) -> Result<(), DrawError> {
        if frame.format != VideoFormat::Y800 {
            return Err(DrawError::WrongFormat);
        }
        if x.saturating_add(w) > frame.width || y.saturating_add(h) > frame.height {
            return Err(DrawError::OutOfBounds);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for row in y..y + h {
                for col in x..x + w {
                    let i = (row * frame.width + col) as usize;
                    if i < plane.len() {
                        plane[i] = lum;
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
        assert!(Draw::rect(&mut f, 3, 3, 2, 2, 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        Draw::rect(&mut f, 1, 1, 2, 2, 200).unwrap();
        assert_eq!(f.planes[0][1 * 4 + 1], 200);
        assert_eq!(f.planes[0][1 * 4 + 2], 200);
        assert_eq!(f.planes[0][2 * 4 + 1], 200);
        assert_eq!(f.planes[0][0], 0);
    }
}
