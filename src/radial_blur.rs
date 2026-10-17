//! Safe pure RadialBlur (center-weighted smear core).
//!
//! Averages each pixel with the pixel one step toward
//! the center. Works on Y800. No unwraps.
//!
//! Differential: a bright edge pixel is reduced.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadialError {
    WrongFormat,
}

pub struct RadialBlur;

impl RadialBlur {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), RadialError> {
        if frame.format != VideoFormat::Y800 {
            return Err(RadialError::WrongFormat);
        }
        let w = frame.width as i32;
        let h = frame.height as i32;
        let cx = w / 2;
        let cy = h / 2;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    let dx = if x > cx { -1 } else if x < cx { 1 } else { 0 };
                    let dy = if y > cy { -1 } else if y < cy { 1 } else { 0 };
                    let nx = (x + dx).clamp(0, w - 1);
                    let ny = (y + dy).clamp(0, h - 1);
                    let ni = (ny * w + nx) as usize;
                    dst[i] = ((src[i] as u16 + src[ni] as u16) / 2) as u8;
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
        RadialBlur::apply(&mut f).unwrap();
        assert!(f.planes[0][0] < 255);
    }
}
