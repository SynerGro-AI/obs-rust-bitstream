//! Safe pure Grid (line grid fill core).
//!
//! Draws horizontal and vertical lines. Works on Y800.
//! No unwraps.
//!
//! Differential: a grid line pixel is set.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GridError {
    WrongFormat,
}

pub struct Grid;

impl Grid {
    /// spacing is pixels between lines. color is the line value.
    pub fn apply(frame: &mut VideoFrame, spacing: u8, color: u8) -> Result<(), GridError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GridError::WrongFormat);
        }
        let step = spacing.max(1) as u32;
        let w = frame.width;
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..frame.height {
                for x in 0..w {
                    if x % step == 0 || y % step == 0 {
                        let i = (y * w + x) as usize;
                        if i < plane.len() {
                            plane[i] = color;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Read spacing from live weights.
    pub fn from_weights(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
        color: u8,
    ) -> Result<(), GridError> {
        let v = weights.get(name).unwrap_or(8.0);
        Self::apply(frame, v.clamp(1.0, 255.0) as u8, color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        Grid::apply(&mut f, 4, 200).unwrap();
        assert_eq!(f.planes[0][0], 200);
        assert_eq!(f.planes[0][8 + 1], 0);
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        let mut w = LiveWeights::new();
        w.set("grid.spacing", 4.0).unwrap();
        Grid::from_weights(&mut f, &w, "grid.spacing", 150).unwrap();
        assert_eq!(f.planes[0][0], 150);
    }
}
