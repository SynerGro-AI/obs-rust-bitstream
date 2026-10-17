//! Safe pure Bevel (edge highlight/shadow core).
//!
//! Brightens the top-left edge and darkens the
//! bottom-right edge of bright regions. Works on Y800.
//! No unwraps.
//!
//! Differential: a bright pixel gains a darker neighbor.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BevelError {
    WrongFormat,
}

pub struct Bevel;

impl Bevel {
    /// depth is 0..=255.
    pub fn apply(frame: &mut VideoFrame, depth: u8) -> Result<(), BevelError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BevelError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    if src[i] > 128 {
                        // Top-left highlight
                        let hi = (y - 1) * w + (x - 1);
                        let add = depth as u16;
                        let v = (dst[hi] as u16 + add).min(255);
                        dst[hi] = v as u8;
                        // Bottom-right shadow
                        let si = (y + 1) * w + (x + 1);
                        if si < dst.len() {
                            let dark = 255u16.saturating_sub(depth as u16);
                            let v = (dst[si] as u16 * dark) / 255;
                            dst[si] = v as u8;
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
        f.planes[0].fill(200);
        Bevel::apply(&mut f, 40).unwrap();
        // Some pixel differs from the flat 200.
        assert!(f.planes[0].iter().any(|p| *p != 200));
    }
}
