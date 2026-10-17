//! Safe pure PaintBucket (flood fill core).
//!
//! Fills a connected region of similar pixels from a
//! seed. Out-of-bounds rejected. Works on Y800.
//! No unwraps.
//!
//! Differential: a connected region is filled, a
//! different region is not.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BucketError {
    WrongFormat,
    OutOfBounds,
}

pub struct PaintBucket;

impl PaintBucket {
    /// Fill from (x, y) where pixels are within tolerance of the seed.
    pub fn fill(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        color: u8,
        tolerance: u8,
    ) -> Result<(), BucketError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BucketError::WrongFormat);
        }
        if x >= frame.width || y >= frame.height {
            return Err(BucketError::OutOfBounds);
        }
        let w = frame.width;
        let h = frame.height;
        let seed = frame.planes[0][(y * w + x) as usize];
        let mut stack = vec![(x, y)];
        let mut seen = vec![false; (w * h) as usize];
        while let Some((cx, cy)) = stack.pop() {
            let i = (cy * w + cx) as usize;
            if seen[i] {
                continue;
            }
            seen[i] = true;
            if frame.planes[0][i].abs_diff(seed) > tolerance {
                continue;
            }
            frame.planes[0][i] = color;
            if cx > 0 {
                stack.push((cx - 1, cy));
            }
            if cx + 1 < w {
                stack.push((cx + 1, cy));
            }
            if cy > 0 {
                stack.push((cx, cy - 1));
            }
            if cy + 1 < h {
                stack.push((cx, cy + 1));
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
        assert!(PaintBucket::fill(&mut f, 5, 5, 255, 10).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(100);
        f.planes[0][3] = 200;
        PaintBucket::fill(&mut f, 0, 0, 50, 10).unwrap();
        assert_eq!(f.planes[0][0], 50);
        assert_eq!(f.planes[0][3], 200);
    }
}
