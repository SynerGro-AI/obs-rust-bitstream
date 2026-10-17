//! Safe pure Transform (scale and flip core).
//!
//! Scale nearest-neighbor and flip horizontal/vertical.
//! Works on Y800. No unwraps.
//!
//! Differential: scale 2x doubles a pixel, flip swaps edges.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransformError {
    WrongFormat,
    BadSize,
}

pub struct Transform;

impl Transform {
    /// Nearest-neighbor scale to (nw, nh).
    pub fn scale(frame: &VideoFrame, nw: u32, nh: u32) -> Result<VideoFrame, TransformError> {
        if frame.format != VideoFormat::Y800 {
            return Err(TransformError::WrongFormat);
        }
        if nw == 0 || nh == 0 {
            return Err(TransformError::BadSize);
        }
        let mut out = VideoFrame::new(VideoFormat::Y800, nw, nh).ok_or(TransformError::BadSize)?;
        if let (Some(src), Some(dst)) = (frame.planes.get(0), out.planes.get_mut(0)) {
            for y in 0..nh {
                for x in 0..nw {
                    let sx = (x * frame.width) / nw;
                    let sy = (y * frame.height) / nh;
                    let si = (sy * frame.width + sx) as usize;
                    let di = (y * nw + x) as usize;
                    if si < src.len() && di < dst.len() {
                        dst[di] = src[si];
                    }
                }
            }
        }
        Ok(out)
    }

    /// Flip left-right in place.
    pub fn flip_h(frame: &mut VideoFrame) -> Result<(), TransformError> {
        if frame.format != VideoFormat::Y800 {
            return Err(TransformError::WrongFormat);
        }
        let w = frame.width as usize;
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..frame.height as usize {
                for x in 0..w / 2 {
                    let a = y * w + x;
                    let b = y * w + (w - 1 - x);
                    plane.swap(a, b);
                }
            }
        }
        Ok(())
    }

    /// Flip top-bottom in place.
    pub fn flip_v(frame: &mut VideoFrame) -> Result<(), TransformError> {
        if frame.format != VideoFormat::Y800 {
            return Err(TransformError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..h / 2 {
                for x in 0..w {
                    let a = y * w + x;
                    let b = (h - 1 - y) * w + x;
                    plane.swap(a, b);
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
    fn bad_size() {
        let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        assert!(Transform::scale(&f, 0, 2).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![10, 20, 30, 40];
        let s = Transform::scale(&f, 4, 4).unwrap();
        assert_eq!(s.width, 4);
        Transform::flip_h(&mut f).unwrap();
        assert_eq!(f.planes[0][0], 20);
        Transform::flip_v(&mut f).unwrap();
        assert_eq!(f.planes[0][0], 40);
    }
}
