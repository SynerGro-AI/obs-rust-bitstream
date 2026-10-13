//! Safe pure Keyer (background removal core).
//!
//! Removes a keyed color from the camera frame so the
//! background animation shows through. No unwraps.
//!
//! Differential: a green sample keys out, a skin sample stays.

#[derive(Clone, Debug)]
pub struct Keyer {
    key_r: f32,
    key_g: f32,
    key_b: f32,
    tolerance: f32,
}

impl Keyer {
    pub fn new() -> Self {
        Self {
            key_r: 0.0,
            key_g: 1.0,
            key_b: 0.0,
            tolerance: 0.3,
        }
    }

    pub fn set_key(&mut self, r: f32, g: f32, b: f32) {
        self.key_r = r;
        self.key_g = g;
        self.key_b = b;
    }

    pub fn set_tolerance(&mut self, t: f32) {
        self.tolerance = t.clamp(0.0, 1.0);
    }

    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    /// True when the sample matches the key color within tolerance.
    pub fn is_background(&self, r: f32, g: f32, b: f32) -> bool {
        let dr = (r - self.key_r).abs();
        let dg = (g - self.key_g).abs();
        let db = (b - self.key_b).abs();
        (dr + dg + db) / 3.0 <= self.tolerance
    }

    /// Composite: keep the subject, replace the background.
    pub fn composite(
        &self,
        camera: (f32, f32, f32),
        background: (f32, f32, f32),
    ) -> (f32, f32, f32) {
        if self.is_background(camera.0, camera.1, camera.2) {
            background
        } else {
            camera
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let k = Keyer::new();
        assert_eq!(k.tolerance(), 0.3);
    }

    #[test]
    fn differential() {
        let k = Keyer::new();
        assert!(k.is_background(0.0, 1.0, 0.0));
        assert!(!k.is_background(0.8, 0.6, 0.5));
        let out = k.composite((0.0, 1.0, 0.0), (0.4, 0.1, 0.5));
        assert_eq!(out, (0.4, 0.1, 0.5));
        let skin = k.composite((0.8, 0.6, 0.5), (0.4, 0.1, 0.5));
        assert_eq!(skin, (0.8, 0.6, 0.5));
    }
}
