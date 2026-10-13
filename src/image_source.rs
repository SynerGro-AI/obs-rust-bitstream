//! Safe pure ImageSource (static image core).
//!
//! Holds a tightly packed image. Empty names rejected.
//! No unwraps.
//!
//! Differential: store + pixel lookup round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageError {
    EmptyName,
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct ImageSource {
    name: String,
    width: usize,
    pixels: Vec<u8>,
}

impl ImageSource {
    pub fn new(name: &str, width: usize, pixels: &[u8]) -> Result<Self, ImageError> {
        if name.is_empty() || width == 0 {
            return Err(ImageError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            width,
            pixels: pixels.to_vec(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.pixels.len() / self.width
    }

    pub fn pixel(&self, x: usize, y: usize) -> Result<u8, ImageError> {
        if x >= self.width || y >= self.height() {
            return Err(ImageError::OutOfBounds);
        }
        Ok(self.pixels[y * self.width + x])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name() {
        assert!(ImageSource::new("", 2, &[1, 2, 3, 4]).is_err());
    }

    #[test]
    fn differential() {
        let img = ImageSource::new("logo", 2, &[1, 2, 3, 4]).unwrap();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.pixel(1, 1).unwrap(), 4);
        assert!(img.pixel(2, 0).is_err());
    }
}
