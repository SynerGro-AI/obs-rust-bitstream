//! Safe pure ColorFilter (brightness / contrast / saturation).
//!
//! Applies brightness, contrast, and saturation to a pixel.
//! No unwraps.
//!
//! Differential: brightness 0.5 lifts a mid gray.

#[derive(Clone, Debug)]
pub struct ColorFilter {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for ColorFilter {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl ColorFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply to one RGB pixel. Returns clamped 0..=255.
    pub fn apply(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;
        let lum = 0.299 * rf + 0.587 * gf + 0.114 * bf;
        let sat = |c: f32| lum + (c - lum) * self.saturation;
        let contrast = |c: f32| (c - 0.5) * self.contrast + 0.5 + self.brightness;
        let clamp = |c: f32| (c.clamp(0.0, 1.0) * 255.0) as u8;
        (
            clamp(contrast(sat(rf))),
            clamp(contrast(sat(gf))),
            clamp(contrast(sat(bf))),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let f = ColorFilter::new();
        assert_eq!(f.apply(128, 128, 128), (128, 128, 128));
    }

    #[test]
    fn differential() {
        let mut f = ColorFilter::new();
        f.brightness = 0.5;
        let (r, _, _) = f.apply(128, 128, 128);
        assert!(r > 128);
    }
}
