//! Safe pure OuterGlow (edge halo core).
//!
//! Brightens pixels just outside a bright region.
//! Works on Y800. No unwraps.
//!
//! Differential: a neighbor of a bright pixel is lifted.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OuterError {
    WrongFormat,
}

pub struct OuterGlow;

impl OuterGlow {
    /// Brighten dark pixels that neighbor a bright one.
    /// strength is 0..=255.
    pub fn apply(frame: &mut VideoFrame, strength: u8) -> Result<(), OuterError> {
        if frame.format != VideoFormat::Y800 {
            return Err(OuterError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    if src[i] < 64 {
                        let neighbors = [src[i - 1], src[i + 1], src[i - w], src[i + w]];
                        if neighbors.iter().any(|n| *n > 180) {
                            let add = strength as u16;
                            let v = (dst[i] as u16 + add).min(255);
                            dst[i] = v as u8;
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
        f.planes[0][1 * 4 + 1] = 255;
        OuterGlow::apply(&mut f, 80).unwrap();
        assert!(f.planes[0][1 * 4 + 2] > 0);
    }
}
