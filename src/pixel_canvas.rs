//! Safe pure PixelCanvas (paint buffer core).
//!
//! Owns a width, a height, and a flat RGB buffer. Empty sizes
//! rejected. No unwraps.
//!
//! Differential: set pixel + get round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelError {
    EmptySize,
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct PixelCanvas {
    width: u32,
    height: u32,
    pixels: Vec<f32>,
}

impl PixelCanvas {
    pub fn new(width: u32, height: u32) -> Result<Self, PixelError> {
        if width == 0 || height == 0 {
            return Err(PixelError::EmptySize);
        }
        let n = (width as usize) * (height as usize) * 3;
        Ok(Self {
            width,
            height,
            pixels: vec![0.0; n],
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn index(&self, x: u32, y: u32) -> Result<usize, PixelError> {
        if x >= self.width || y >= self.height {
            return Err(PixelError::OutOfBounds);
        }
        Ok(((y as usize) * (self.width as usize) + (x as usize)) * 3)
    }

    pub fn set(&mut self, x: u32, y: u32, r: f32, g: f32, b: f32) -> Result<(), PixelError> {
        let i = self.index(x, y)?;
        self.pixels[i] = r.clamp(0.0, 1.0);
        self.pixels[i + 1] = g.clamp(0.0, 1.0);
        self.pixels[i + 2] = b.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn get(&self, x: u32, y: u32) -> Result<(f32, f32, f32), PixelError> {
        let i = self.index(x, y)?;
        Ok((self.pixels[i], self.pixels[i + 1], self.pixels[i + 2]))
    }

    pub fn fill(&mut self, r: f32, g: f32, b: f32) {
        for i in (0..self.pixels.len()).step_by(3) {
            self.pixels[i] = r.clamp(0.0, 1.0);
            self.pixels[i + 1] = g.clamp(0.0, 1.0);
            self.pixels[i + 2] = b.clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = PixelCanvas::new(4, 4).unwrap();
        assert_eq!(c.width(), 4);
    }

    #[test]
    fn empty_size() {
        assert!(PixelCanvas::new(0, 4).is_err());
    }

    #[test]
    fn differential() {
        let mut c = PixelCanvas::new(8, 8).unwrap();
        c.set(2, 3, 1.0, 0.0, 0.0).unwrap();
        let (r, g, b) = c.get(2, 3).unwrap();
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert!(c.get(9, 0).is_err());
    }
}
