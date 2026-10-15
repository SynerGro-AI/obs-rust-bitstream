//! Safe pure BrightShadow (highlight shadow core).
//!
//! Offsets a bright region and lightens the offset copy.
//! Works on Y800. No unwraps.
//!
//! Differential: a white pixel gains a bright neighbor.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrightError {
    WrongFormat,
}

pub struct BrightShadow;

impl BrightShadow {
    /// Lighten pixels offset by (dx, dy) where the source is bright.
    /// strength is 0..=255 (how bright the highlight is).
    pub fn apply(
        frame: &mut VideoFrame,
        dx: i32,
        dy: i32,
        strength: u8,
    ) -> Result<(), BrightError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BrightError::WrongFormat);
        }
        let w = frame.width as i32;
        let h = frame.height as i32;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 0..h {
                for x in 0..w {
                    let sx = x - dx;
                    let sy = y - dy;
                    if sx >= 0 && sx < w && sy >= 0 && sy < h {
                        let si = (sy * w + sx) as usize;
                        if src[si] > 128 {
                            let di = (y * w + x) as usize;
                            if di < dst.len() {
                                let add = strength as u16;
                                let v = (dst[di] as u16 + add).min(255);
                                dst[di] = v as u8;
                            }
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
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0][0] = 255;
        BrightShadow::apply(&mut f, 1, 0, 100).unwrap();
        assert!(f.planes[0][1] > 0);
    }
}
