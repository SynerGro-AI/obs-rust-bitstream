//! Safe pure QuickSelect (grow-from-seed core).
//!
//! Starts at a seed pixel and grows to neighbors within
//! tolerance. Works with Selection and a Y800 frame.
//! No unwraps.
//!
//! Differential: a connected region is selected, a
//! distant different region is not.

use crate::pixel_selection::{SelectError, Selection};
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuickError {
    OutOfBounds,
    SizeMismatch,
}

impl From<SelectError> for QuickError {
    fn from(_: SelectError) -> Self {
        QuickError::SizeMismatch
    }
}

pub struct QuickSelect;

impl QuickSelect {
    pub fn select(
        sel: &mut Selection,
        frame: &VideoFrame,
        seed_x: u32,
        seed_y: u32,
        tolerance: u8,
    ) -> Result<(), QuickError> {
        if frame.width != sel.width() || frame.height != sel.height() {
            return Err(QuickError::SizeMismatch);
        }
        if seed_x >= sel.width() || seed_y >= sel.height() {
            return Err(QuickError::OutOfBounds);
        }
        let plane = &frame.planes[0];
        let seed = plane[(seed_y * sel.width() + seed_x) as usize];
        let mut stack = vec![(seed_x, seed_y)];
        let mut seen = vec![false; plane.len()];
        while let Some((x, y)) = stack.pop() {
            let i = (y * sel.width() + x) as usize;
            if seen[i] {
                continue;
            }
            seen[i] = true;
            if plane[i].abs_diff(seed) > tolerance {
                continue;
            }
            let _ = sel.rect(x, y, 1, 1);
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < sel.width() {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < sel.height() {
                stack.push((x, y + 1));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(100);
        f.planes[0][3] = 200;
        let mut s = Selection::new(4, 4);
        QuickSelect::select(&mut s, &f, 0, 0, 10).unwrap();
        assert!(s.is_selected(0, 0));
        assert!(s.is_selected(1, 0));
        assert!(!s.is_selected(3, 0));
    }
}
