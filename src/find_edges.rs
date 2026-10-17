//! Safe pure FindEdges (gradient magnitude core).
//!
//! Replaces each interior pixel with the absolute
//! difference to its right and lower neighbors.
//! Works on Y800. No unwraps.
//!
//! Differential: a flat area stays low, an edge is high.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EdgesError {
    WrongFormat,
}

pub struct FindEdges;

impl FindEdges {
    pub fn apply(frame: &mut VideoFrame) -> Result<(), EdgesError> {
        if frame.format != VideoFormat::Y800 {
            return Err(EdgesError::WrongFormat);
        }
        let w = frame.width as usize;
        let h = frame.height as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 0..h.saturating_sub(1) {
                for x in 0..w.saturating_sub(1) {
                    let i = y * w + x;
                    let dx = src[i].abs_diff(src[i + 1]);
                    let dy = src[i].abs_diff(src[i + w]);
                    dst[i] = dx.max(dy);
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
        FindEdges::apply(&mut f).unwrap();
        assert!(f.planes[0][1 * 4 + 1] > 100);
        assert_eq!(f.planes[0][0], 0);
    }
}
