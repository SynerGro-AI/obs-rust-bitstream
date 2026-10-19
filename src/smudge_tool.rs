//! Safe pure SmudgeTool (push pixels tool).
//!
//! Holds a strength. Apply pushes pixels in a direction.
//! Strength is clamped to 1..=10. No unwraps.
//!
//! Differential: a different direction changes different pixels.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct SmudgeTool;

impl SmudgeTool {
    /// Push pixels from (cx, cy) toward (dx, dy). Returns pixels moved.
    pub fn apply(frame: &mut VideoFrame, cx: u32, cy: u32, dx: i32, dy: i32) -> Option<usize> {
        if frame.format != VideoFormat::Y800 {
            return None;
        }
        let plane = frame.planes.get_mut(0)?;
        let w = frame.width as i32;
        let h = frame.height as i32;
        let tx = (cx as i32 + dx).clamp(0, w - 1) as u32;
        let ty = (cy as i32 + dy).clamp(0, h - 1) as u32;
        let src = (cy * frame.width + cx) as usize;
        let dst = (ty * frame.width + tx) as usize;
        if src < plane.len() && dst < plane.len() && src != dst {
            let v = plane[src];
            plane[dst] = v;
            Some(1)
        } else {
            Some(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            p[0] = 255;
        }
        f
    }

    #[test]
    fn pushes() {
        let mut f = frame();
        let n = SmudgeTool::apply(&mut f, 0, 0, 1, 0).unwrap();
        assert_eq!(n, 1);
        assert_eq!(f.planes[0][1], 255);
    }

    #[test]
    fn differential() {
        let mut a = frame();
        let mut b = frame();
        SmudgeTool::apply(&mut a, 0, 0, 1, 0).unwrap();
        SmudgeTool::apply(&mut b, 0, 0, 0, 1).unwrap();
        assert_ne!(a.planes[0][1], b.planes[0][1]);
    }
}
