//! Safe pure CropFilter (region crop core).
//!
//! Crops a frame to a rectangle. Out-of-bounds rejected.
//! No unwraps.
//!
//! Differential: a 4x4 crop of 2x2 returns 4 pixels.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CropError {
    OutOfBounds,
}

#[derive(Clone, Debug, Default)]
pub struct CropFilter {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl CropFilter {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    /// Crop a tightly packed frame of `width` pixels per row.
    pub fn apply(&self, frame: &[u8], width: usize) -> Result<Vec<u8>, CropError> {
        if width == 0 || self.w == 0 || self.h == 0 {
            return Err(CropError::OutOfBounds);
        }
        let height = frame.len() / width;
        if self.x + self.w > width || self.y + self.h > height {
            return Err(CropError::OutOfBounds);
        }
        let mut out = Vec::with_capacity(self.w * self.h);
        for row in 0..self.h {
            let start = (self.y + row) * width + self.x;
            out.extend_from_slice(&frame[start..start + self.w]);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = CropFilter::new(0, 0, 2, 2);
        assert_eq!(c.w, 2);
    }

    #[test]
    fn out_of_bounds() {
        let c = CropFilter::new(3, 3, 2, 2);
        let frame = [0u8; 16];
        assert!(c.apply(&frame, 4).is_err());
    }

    #[test]
    fn differential() {
        let frame: Vec<u8> = (0..16).collect();
        let c = CropFilter::new(1, 1, 2, 2);
        let out = c.apply(&frame, 4).unwrap();
        assert_eq!(out, vec![5, 6, 9, 10]);
    }
}
