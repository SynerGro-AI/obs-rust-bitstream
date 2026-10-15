//! Safe pure DropShadow (dark shadow core).
//!
//! Offsets a bright region and darkens the offset copy.
//! Works on Y800. No unwraps.
//!
//! Differential: a white pixel gains a dark neighbor.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadowError {
    WrongFormat,
}

pub struct DropShadow;

impl DropShadow {
    /// Darken pixels offset by (dx, dy) where the source is bright.
    /// strength is 0..=255 (how dark the shadow is).
    pub fn apply(
        frame: &mut VideoFrame,
        dx: i32,
        dy: i32,
        strength: u8,
    ) -> Result<(), ShadowError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ShadowError::WrongFormat);
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
                                let dark = 255u16.saturating_sub(strength as u16);
                                let v = (dst[di] as u16 * dark) / 255;
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
        DropShadow::apply(&mut f, 1, 1, 200).unwrap();
        // Pixel at (1,1) should be darkened
        assert!(f.planes[0][1 * 4 + 1] < 128);
    }
}
