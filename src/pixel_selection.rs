//! Safe pure Selection (pixel mask core).
//!
//! Holds a boolean mask. Supports rect, invert, and
//! magic-wand (tolerance). No unwraps.
//!
//! Differential: invert flips a selected pixel.

use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectError {
    SizeMismatch,
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct Selection {
    width: u32,
    height: u32,
    mask: Vec<bool>,
}

impl Selection {
    pub fn new(width: u32, height: u32) -> Self {
        let n = (width as usize) * (height as usize);
        Self {
            width,
            height,
            mask: vec![false; n],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), SelectError> {
        if x.saturating_add(w) > self.width || y.saturating_add(h) > self.height {
            return Err(SelectError::OutOfBounds);
        }
        for row in y..y + h {
            for col in x..x + w {
                let i = (row * self.width + col) as usize;
                self.mask[i] = true;
            }
        }
        Ok(())
    }

    pub fn invert(&mut self) {
        for m in self.mask.iter_mut() {
            *m = !*m;
        }
    }

    /// Select pixels within tolerance of the seed color.
    pub fn magic_wand(
        &mut self,
        frame: &VideoFrame,
        seed_x: u32,
        seed_y: u32,
        tolerance: u8,
    ) -> Result<(), SelectError> {
        if frame.width != self.width || frame.height != self.height {
            return Err(SelectError::SizeMismatch);
        }
        if seed_x >= self.width || seed_y >= self.height {
            return Err(SelectError::OutOfBounds);
        }
        let plane = &frame.planes[0];
        let seed = plane[(seed_y * self.width + seed_x) as usize];
        for (i, p) in plane.iter().enumerate() {
            if p.abs_diff(seed) <= tolerance {
                self.mask[i] = true;
            }
        }
        Ok(())
    }

    pub fn is_selected(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.mask[(y * self.width + x) as usize]
    }

    pub fn count(&self) -> usize {
        self.mask.iter().filter(|m| **m).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn invert() {
        let mut s = Selection::new(2, 2);
        s.rect(0, 0, 1, 1).unwrap();
        assert!(s.is_selected(0, 0));
        s.invert();
        assert!(!s.is_selected(0, 0));
        assert!(s.is_selected(1, 1));
    }

    #[test]
    fn magic() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![100, 100, 200, 200];
        let mut s = Selection::new(2, 2);
        s.magic_wand(&f, 0, 0, 10).unwrap();
        assert_eq!(s.count(), 2);
    }
}
