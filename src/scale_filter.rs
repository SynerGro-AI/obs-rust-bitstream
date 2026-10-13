//! Safe pure ScaleFilter (nearest-neighbor scale core).
//!
//! Scales a frame to a new size. Zero size rejected.
//! No unwraps.
//!
//! Differential: a 2x2 scaled to 4x4 repeats each pixel.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScaleError {
    ZeroSize,
}

#[derive(Clone, Debug)]
pub struct ScaleFilter {
    pub dst_w: usize,
    pub dst_h: usize,
}

impl ScaleFilter {
    pub fn new(dst_w: usize, dst_h: usize) -> Self {
        Self { dst_w, dst_h }
    }

    /// Nearest-neighbor scale. `src` is tightly packed at `src_w`.
    pub fn apply(&self, src: &[u8], src_w: usize) -> Result<Vec<u8>, ScaleError> {
        if src_w == 0 || self.dst_w == 0 || self.dst_h == 0 {
            return Err(ScaleError::ZeroSize);
        }
        let src_h = src.len() / src_w;
        if src_h == 0 {
            return Err(ScaleError::ZeroSize);
        }
        let mut out = Vec::with_capacity(self.dst_w * self.dst_h);
        for dy in 0..self.dst_h {
            let sy = dy * src_h / self.dst_h;
            for dx in 0..self.dst_w {
                let sx = dx * src_w / self.dst_w;
                out.push(src[sy * src_w + sx]);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = ScaleFilter::new(4, 4);
        assert_eq!(s.dst_w, 4);
    }

    #[test]
    fn zero() {
        let s = ScaleFilter::new(0, 4);
        assert!(s.apply(&[1, 2, 3, 4], 2).is_err());
    }

    #[test]
    fn differential() {
        let src = [1, 2, 3, 4];
        let s = ScaleFilter::new(4, 4);
        let out = s.apply(&src, 2).unwrap();
        assert_eq!(out.len(), 16);
        assert_eq!(out[0], 1);
        assert_eq!(out[1], 1);
        assert_eq!(out[2], 2);
        assert_eq!(out[15], 4);
    }
}
