//! Safe pure BlurTool (soften tool).
//!
//! Holds a radius. Apply softens a region of a frame.
//! Radius is clamped to 1..=20. No unwraps.
//!
//! Differential: a larger radius changes more pixels.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct BlurTool;

impl BlurTool {
    /// Soften a square region centered at (cx, cy) with the given radius.
    /// Returns the number of pixels changed. None if the format is wrong.
    pub fn apply(frame: &mut VideoFrame, cx: u32, cy: u32, radius: u32) -> Option<usize> {
        if frame.format != VideoFormat::Y800 {
            return None;
        }
        let r = radius.clamp(1, 20);
        let plane = frame.planes.get_mut(0)?;
        let w = frame.width;
        let h = frame.height;
        let mut count = 0;
        let x0 = cx.saturating_sub(r);
        let y0 = cy.saturating_sub(r);
        let x1 = (cx + r).min(w);
        let y1 = (cy + r).min(h);
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = (y * w + x) as usize;
                if let Some(p) = plane.get_mut(idx) {
                    *p = (*p as u16 * 3 / 4) as u8;
                    count += 1;
                }
            }
        }
        Some(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = 200;
            }
        }
        f
    }

    #[test]
    fn softens() {
        let mut f = frame();
        let n = BlurTool::apply(&mut f, 8, 8, 2).unwrap();
        assert!(n > 0);
        assert!(f.planes[0][8 * 16 + 8] < 200);
    }

    #[test]
    fn differential() {
        let mut a = frame();
        let mut b = frame();
        let na = BlurTool::apply(&mut a, 8, 8, 1).unwrap();
        let nb = BlurTool::apply(&mut b, 8, 8, 3).unwrap();
        assert!(nb > na);
    }
}
