//! Safe pure Sharpen (simple sharpen core).
//!
//! Boosts a pixel relative to its neighbors. Works on Y800.
//! No unwraps.
//!
//! Differential: a bright pixel on dark is lifted further.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharpenError {
    WrongFormat,
}

pub struct Sharpen;

impl Sharpen {
    /// amount is 0..=255. Higher boosts more.
    pub fn apply(frame: &mut VideoFrame, amount: u8) -> Result<(), SharpenError> {
        if frame.format != VideoFormat::Y800 {
            return Err(SharpenError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    let neighbors = (src[i - 1] as u16
                        + src[i + 1] as u16
                        + src[i - w] as u16
                        + src[i + w] as u16)
                        / 4;
                    let diff = src[i] as i16 - neighbors as i16;
                    let boost = (diff * amount as i16) / 128;
                    let v = (src[i] as i16 + boost).clamp(0, 255);
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
        f.planes[0].fill(32);
        f.planes[0][1 * 4 + 1] = 200;
        Sharpen::apply(&mut f, 128).unwrap();
        assert!(f.planes[0][1 * 4 + 1] >= 200);
    }
}
