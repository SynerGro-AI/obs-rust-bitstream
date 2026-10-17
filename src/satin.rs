//! Safe pure Satin (offset invert overlay core).
//!
//! Offsets a copy, inverts it, and blends it back.
//! Works on Y800. No unwraps.
//!
//! Differential: a bright pixel changes after satin.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SatinError {
    WrongFormat,
}

pub struct Satin;

impl Satin {
    /// distance is the offset in pixels.
    pub fn apply(frame: &mut VideoFrame, distance: u8) -> Result<(), SatinError> {
        if frame.format != VideoFormat::Y800 {
            return Err(SatinError::WrongFormat);
        }
        let w = frame.width as i32;
        let h = frame.height as i32;
        let d = distance.max(1) as i32;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    let sx = (x - d).clamp(0, w - 1);
                    let sy = (y - d).clamp(0, h - 1);
                    let si = (sy * w + sx) as usize;
                    let inv = 255 - src[si];
                    let v = (src[i] as u16 + inv as u16) / 2;
                    dst[i] = v as u8;
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
        Satin::apply(&mut f, 1).unwrap();
        assert!(f.planes[0].iter().any(|p| *p != 200));
    }
}
