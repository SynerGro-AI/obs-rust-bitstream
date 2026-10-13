//! Safe pure ColorSource (solid color core).
//!
//! Holds an RGB color. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorSource {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorSource {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    pub fn set(&mut self, r: u8, g: u8, b: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    /// Hex string like "#ff8800".
    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = ColorSource::new(255, 128, 0);
        assert_eq!(c.rgb(), (255, 128, 0));
    }

    #[test]
    fn differential() {
        let mut c = ColorSource::new(0, 0, 0);
        c.set(255, 136, 0);
        assert_eq!(c.hex(), "#ff8800");
    }
}
