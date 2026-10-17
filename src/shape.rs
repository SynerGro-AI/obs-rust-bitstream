//! Safe pure Shape (rect and ellipse core).
//!
//! Fills a rectangle or ellipse into a Y800 frame.
//! Out-of-bounds rejected. No unwraps.
//!
//! Differential: a rect sets its pixels, an ellipse
//! sets the center.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    WrongFormat,
    OutOfBounds,
}

pub struct Shape;

impl Shape {
    pub fn rect(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: u8,
    ) -> Result<(), ShapeError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ShapeError::WrongFormat);
        }
        if x.saturating_add(w) > frame.width || y.saturating_add(h) > frame.height {
            return Err(ShapeError::OutOfBounds);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for row in y..y + h {
                for col in x..x + w {
                    let i = (row * frame.width + col) as usize;
                    if i < plane.len() {
                        plane[i] = color;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn ellipse(
        frame: &mut VideoFrame,
        cx: u32,
        cy: u32,
        rx: u32,
        ry: u32,
        color: u8,
    ) -> Result<(), ShapeError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ShapeError::WrongFormat);
        }
        if rx == 0 || ry == 0 {
            return Err(ShapeError::OutOfBounds);
        }
        let w = frame.width as i32;
        let h = frame.height as i32;
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..h {
                for x in 0..w {
                    let dx = (x - cx as i32) as f32 / rx as f32;
                    let dy = (y - cy as i32) as f32 / ry as f32;
                    if dx * dx + dy * dy <= 1.0 {
                        let i = (y as u32 * frame.width + x as u32) as usize;
                        if i < plane.len() {
                            plane[i] = color;
                        }
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
        assert!(Shape::rect(&mut f, 3, 3, 2, 2, 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        Shape::rect(&mut f, 1, 1, 2, 2, 200).unwrap();
        assert_eq!(f.planes[0][1 * 8 + 1], 200);
        Shape::ellipse(&mut f, 4, 4, 2, 2, 100).unwrap();
        assert_eq!(f.planes[0][4 * 8 + 4], 100);
    }
}
