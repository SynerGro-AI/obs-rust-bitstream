//! Safe pure Transform (position, scale, rotation core).
//!
//! A transform owns a translation, a uniform scale, and a
//! rotation in radians. No unwraps.
//!
//! Differential: set + get round-trips.

#[derive(Clone, Debug)]
pub struct Transform {
    x: f32,
    y: f32,
    scale: f32,
    rotation: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn set_scale(&mut self, s: f32) {
        self.scale = s.max(0.0);
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_rotation(&mut self, r: f32) {
        self.rotation = r;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Apply the transform to a point: scale, rotate, translate.
    pub fn apply(&self, px: f32, py: f32) -> (f32, f32) {
        let c = self.rotation.cos();
        let s = self.rotation.sin();
        let sx = px * self.scale;
        let sy = py * self.scale;
        let rx = sx * c - sy * s;
        let ry = sx * s + sy * c;
        (rx + self.x, ry + self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = Transform::new();
        assert_eq!(t.scale(), 1.0);
    }

    #[test]
    fn differential() {
        let mut t = Transform::new();
        t.set_position(10.0, 20.0);
        t.set_scale(2.0);
        t.set_rotation(0.0);
        let (x, y) = t.apply(1.0, 0.0);
        assert!((x - 12.0).abs() < 0.001);
        assert!((y - 20.0).abs() < 0.001);
    }
}
