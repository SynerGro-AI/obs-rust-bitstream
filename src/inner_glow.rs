//! Safe pure InnerGlow (edge brighten core).
//!
//! Brightens pixels just inside a dark-to-bright edge.
//! Works on Y800. No unwraps.
//!
//! Differential: an edge pixel is lifted.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlowError {
    WrongFormat,
}

pub struct InnerGlow;

impl InnerGlow {
    /// Brighten pixels that have a darker neighbor.
    /// strength is 0..=255.
    pub fn apply(frame: &mut VideoFrame, strength: u8) -> Result<(), GlowError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GlowError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    let center = src[i];
                    let left = src[i - 1];
                    let up = src[i - w];
                    if center > left.saturating_add(20) || center > up.saturating_add(20) {
                        let add = strength as u16;
                        let v = (dst[i] as u16 + add).min(255);
                        dst[i] = v as u8;
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
        f.planes[0].fill(32);
        f.planes[0][4 + 1] = 200;
        InnerGlow::apply(&mut f, 50).unwrap();
        assert!(f.planes[0][4 + 1] > 200);
    }
}
