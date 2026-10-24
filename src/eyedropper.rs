//! Safe pure Eyedropper (color sample tool).
//!
//! Samples a color from a frame at a point.
//! Out-of-bounds returns None. No unwraps.
//!
//! Differential: a different point samples a different color.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct Eyedropper;

impl Eyedropper {
    /// Sample the Y800 value at (x, y). None if out of bounds.
    pub fn sample(frame: &VideoFrame, x: u32, y: u32) -> Option<u8> {
        if frame.format != VideoFormat::Y800 {
            return None;
        }
        if x >= frame.width || y >= frame.height {
            return None;
        }
        let plane = frame.planes.first()?;
        let idx = (y * frame.width + x) as usize;
        plane.get(idx).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    #[test]
    fn samples() {
        let f = frame(128);
        assert_eq!(Eyedropper::sample(&f, 1, 1), Some(128));
    }

    #[test]
    fn out_of_bounds() {
        let f = frame(128);
        assert!(Eyedropper::sample(&f, 10, 1).is_none());
    }

    #[test]
    fn differential() {
        let a = frame(100);
        let b = frame(200);
        assert_ne!(Eyedropper::sample(&a, 0, 0), Eyedropper::sample(&b, 0, 0));
    }
}
