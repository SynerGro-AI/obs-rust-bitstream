//! Safe pure IndexFrame (4-bit color indices).
//!
//! Holds one index per pixel (0..=15). Zero size rejected.
//! No unwraps.
//!
//! Differential: set + get round-trips.

use crate::palette16::Palette16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexError {
    ZeroSize,
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct IndexFrame {
    width: u32,
    height: u32,
    indices: Vec<u8>,
}

impl IndexFrame {
    pub fn new(width: u32, height: u32) -> Result<Self, IndexError> {
        if width == 0 || height == 0 {
            return Err(IndexError::ZeroSize);
        }
        let n = (width as usize) * (height as usize);
        Ok(Self {
            width,
            height,
            indices: vec![0; n],
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set(&mut self, x: u32, y: u32, index: u8) -> Result<(), IndexError> {
        if x >= self.width || y >= self.height {
            return Err(IndexError::OutOfBounds);
        }
        let i = (y * self.width + x) as usize;
        self.indices[i] = index & 15;
        Ok(())
    }

    pub fn get(&self, x: u32, y: u32) -> Result<u8, IndexError> {
        if x >= self.width || y >= self.height {
            return Err(IndexError::OutOfBounds);
        }
        Ok(self.indices[(y * self.width + x) as usize])
    }

    /// Expand to Y800 using the palette luminance.
    pub fn to_y800(&self) -> Vec<u8> {
        self.indices
            .iter()
            .map(|i| Palette16::luminance(*i))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert!(IndexFrame::new(0, 10).is_err());
    }

    #[test]
    fn differential() {
        let mut f = IndexFrame::new(2, 2).unwrap();
        f.set(1, 1, 15).unwrap();
        assert_eq!(f.get(1, 1).unwrap(), 15);
        assert_eq!(f.get(0, 0).unwrap(), 0);
        let y = f.to_y800();
        assert!(y[3] > y[0]);
    }
}
