//! Safe pure Stroke (outline core).
//!
//! Draws a one-pixel outline around bright regions.
//! Works on Y800. No unwraps.
//!
//! Differential: a bright pixel gains an outlined neighbor.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrokeError {
    WrongFormat,
}

pub struct Stroke;

impl Stroke {
    /// color is the outline luminance. width is 1 for now.
    pub fn apply(frame: &mut VideoFrame, color: u8) -> Result<(), StrokeError> {
        if frame.format != VideoFormat::Y800 {
            return Err(StrokeError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let i = y * w + x;
                    if src[i] > 180 {
                        for (dy, dx) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            let ni = ny * w + nx;
                            if ni < dst.len() && src[ni] < 64 {
                                dst[ni] = color;
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
        f.planes[0][4 + 1] = 255;
        Stroke::apply(&mut f, 200).unwrap();
        assert_eq!(f.planes[0][4 + 2], 200);
    }
}
