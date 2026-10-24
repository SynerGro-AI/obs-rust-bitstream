//! Safe pure PatternSource (test pattern core).
//!
//! Generates per-pixel values for bars or a gradient.
//! Zero size rejected. No unwraps.
//!
//! Differential: a 4-pixel bar pattern returns the
//! expected sequence.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternError {
    ZeroSize,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Bars,
    Gradient,
}

#[derive(Clone, Debug)]
pub struct PatternSource {
    kind: PatternKind,
    width: u32,
    height: u32,
}

impl PatternSource {
    pub fn new(kind: PatternKind, width: u32, height: u32) -> Result<Self, PatternError> {
        if width == 0 || height == 0 {
            return Err(PatternError::ZeroSize);
        }
        Ok(Self {
            kind,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Luminance at pixel (x, y). 0..=255.
    pub fn sample(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        match self.kind {
            PatternKind::Bars => {
                let bar = (x * 4) / self.width;
                (bar * 64) as u8
            }
            PatternKind::Gradient => {
                let v = (x * 255) / self.width.max(1);
                v as u8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert!(PatternSource::new(PatternKind::Bars, 0, 10).is_err());
    }

    #[test]
    fn differential() {
        let p = PatternSource::new(PatternKind::Bars, 4, 1).unwrap();
        assert_eq!(p.sample(0, 0), 0);
        assert_eq!(p.sample(1, 0), 64);
        assert_eq!(p.sample(2, 0), 128);
        assert_eq!(p.sample(3, 0), 192);
    }
}
