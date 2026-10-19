//! Safe pure SharpenTool (edge tool).
//!
//! Holds a strength. Apply increases contrast in a region.
//! Strength is clamped to 1..=10. No unwraps.
//!
//! Differential: a higher strength changes more.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct SharpenTool;

impl SharpenTool {
    /// Increase contrast in a square region. Returns pixels changed.
    pub fn apply(frame: &mut VideoFrame, cx: u32, cy: u32, strength: u32) -> Option<usize> {
        if frame.format != VideoFormat::Y800 {
            return None;
        }
        let s = strength.clamp(1, 10) as i16;
        let plane = frame.planes.get_mut(0)?;
        let w = frame.width;
        let h = frame.height;
        let mut count = 0;
        let x0 = cx.saturating_sub(2);
        let y0 = cy.saturating_sub(2);
        let x1 = (cx + 2).min(w);
        let y1 = (cy + 2).min(h);
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = (y * w + x) as usize;
                if let Some(p) = plane.get_mut(idx) {
                    let v = *p as i16;
                    let shifted = if v > 128 { v + s } else { v - s };
                    *p = shifted.clamp(0, 255) as u8;
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
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for (i, px) in p.iter_mut().enumerate() {
                *px = if i % 2 == 0 { 200 } else { 50 };
            }
        }
        f
    }

    #[test]
    fn sharpens() {
        let mut f = frame();
        let n = SharpenTool::apply(&mut f, 4, 4, 3).unwrap();
        assert!(n > 0);
    }

    #[test]
    fn differential() {
        let mut a = frame();
        let mut b = frame();
        SharpenTool::apply(&mut a, 4, 4, 1).unwrap();
        SharpenTool::apply(&mut b, 4, 4, 5).unwrap();
        assert_ne!(a.planes[0][4 * 8 + 4], b.planes[0][4 * 8 + 4]);
    }
}
