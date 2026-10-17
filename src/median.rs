//! Safe pure Median (3x3 median core).
//!
//! Replaces each interior pixel with the median of its
//! 3x3 neighborhood. Works on Y800. No unwraps.
//!
//! Differential: a lone spike is removed.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MedianError {
    WrongFormat,
}

pub struct Median;

impl Median {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), MedianError> {
        if frame.format != VideoFormat::Y800 {
            return Err(MedianError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 1..h.saturating_sub(1) {
                for x in 1..w.saturating_sub(1) {
                    let mut n = [0u8; 9];
                    let mut k = 0;
                    for dy in 0..3 {
                        for dx in 0..3 {
                            let i = (y + dy - 1) * w + (x + dx - 1);
                            n[k] = src[i];
                            k += 1;
                        }
                    }
                    n.sort_unstable();
                    dst[y * w + x] = n[4];
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
        f.planes[0][1 * 4 + 1] = 255;
        Median::apply(&mut f).unwrap();
        assert_eq!(f.planes[0][1 * 4 + 1], 32);
    }
}
