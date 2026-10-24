//! Safe pure Crop (frame cut core).
//!
//! Returns a new frame containing only the rectangle.
//! Out-of-bounds rejected. Works on Y800. No unwraps.
//!
//! Differential: a 2x2 crop from a larger frame matches.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CropError {
    WrongFormat,
    OutOfBounds,
}

pub struct Crop;

impl Crop {
    pub fn apply(
        frame: &VideoFrame,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<VideoFrame, CropError> {
        if frame.format != VideoFormat::Y800 {
            return Err(CropError::WrongFormat);
        }
        if w == 0
            || h == 0
            || x.saturating_add(w) > frame.width
            || y.saturating_add(h) > frame.height
        {
            return Err(CropError::OutOfBounds);
        }
        let mut out = VideoFrame::new(VideoFormat::Y800, w, h).ok_or(CropError::OutOfBounds)?;
        if let (Some(src), Some(dst)) = (frame.planes.first(), out.planes.get_mut(0)) {
            for row in 0..h {
                for col in 0..w {
                    let si = ((y + row) * frame.width + (x + col)) as usize;
                    let di = (row * w + col) as usize;
                    if si < src.len() && di < dst.len() {
                        dst[di] = src[si];
                    }
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_bounds() {
        let f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        assert!(Crop::apply(&f, 3, 3, 2, 2).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0][4 + 1] = 200;
        let c = Crop::apply(&f, 1, 1, 2, 2).unwrap();
        assert_eq!(c.width, 2);
        assert_eq!(c.height, 2);
        assert_eq!(c.planes[0][0], 200);
    }
}
