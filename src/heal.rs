//! Safe pure Heal (blend-from-offset core).
//!
//! Copies a square from a source offset and blends it
//! with the destination. Out-of-bounds rejected.
//! Works on Y800. No unwraps.
//!
//! Differential: a source pixel is blended into the destination.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealError {
    WrongFormat,
    OutOfBounds,
}

pub struct Heal;

impl Heal {
    /// Blend a size x size square from (sx, sy) into (dx, dy).
    /// amount is 0..=255 (255 is full copy).
    pub fn apply(
        frame: &mut VideoFrame,
        sx: u32,
        sy: u32,
        dx: u32,
        dy: u32,
        size: u8,
        amount: u8,
    ) -> Result<(), HealError> {
        if frame.format != VideoFormat::Y800 {
            return Err(HealError::WrongFormat);
        }
        let s = size.max(1) as u32;
        if sx.saturating_add(s) > frame.width
            || sy.saturating_add(s) > frame.height
            || dx.saturating_add(s) > frame.width
            || dy.saturating_add(s) > frame.height
        {
            return Err(HealError::OutOfBounds);
        }
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for row in 0..s {
                for col in 0..s {
                    let si = ((sy + row) * frame.width + (sx + col)) as usize;
                    let di = ((dy + row) * frame.width + (dx + col)) as usize;
                    if si < src.len() && di < dst.len() {
                        let blended = (src[si] as u16 * amount as u16
                            + dst[di] as u16 * (255 - amount as u16))
                            / 255;
                        dst[di] = blended as u8;
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
        assert!(Heal::apply(&mut f, 3, 3, 0, 0, 2, 128).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0][0] = 200;
        f.planes[0][2 * 4 + 2] = 0;
        Heal::apply(&mut f, 0, 0, 2, 2, 1, 128).unwrap();
        let v = f.planes[0][2 * 4 + 2];
        assert!(v > 0 && v < 200);
    }
}
