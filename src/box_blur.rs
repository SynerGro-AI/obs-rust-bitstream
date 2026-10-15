//! Safe pure BoxBlur (simple blur core).
//!
//! Averages a pixel with its 4-neighbors. Works on Y800.
//! No unwraps.
//!
//! Differential: a lone bright pixel is reduced.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlurError {
    WrongFormat,
}

pub struct BoxBlur;

impl BoxBlur {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), BlurError> {
        if frame.format != VideoFormat::Y800 {
            return Err(BlurError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    let sum = src[i] as u16
                        + src[i - 1] as u16
                        + src[i + 1] as u16
                        + src[i - w] as u16
                        + src[i + w] as u16;
                    dst[i] = (sum / 5) as u8;
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
        BoxBlur::apply(&mut f).unwrap();
        assert!(f.planes[0][1 * 4 + 1] < 255);
    }
}
