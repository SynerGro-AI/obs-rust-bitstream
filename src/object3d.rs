//! Safe pure Object3D (3D object edit core).
//!
//! A 3D object owns a name, a position, a scale, and a
//! rotation. It can be projected onto a pixel canvas.
//! Empty names rejected. No unwraps.
//!
//! Differential: set position + project round-trips.

use crate::pixel_canvas::PixelCanvas;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Object3D {
    name: String,
    x: f32,
    y: f32,
    z: f32,
    scale: f32,
    rot_y: f32,
}

impl Object3D {
    pub fn new(name: &str) -> Result<Self, ObjectError> {
        if name.is_empty() {
            return Err(ObjectError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            x: 0.0,
            y: 0.0,
            z: 1.0,
            scale: 1.0,
            rot_y: 0.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z.max(0.01);
    }

    pub fn position(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    pub fn set_scale(&mut self, s: f32) {
        self.scale = s.max(0.0);
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_rotation_y(&mut self, r: f32) {
        self.rot_y = r;
    }

    pub fn rotation_y(&self) -> f32 {
        self.rot_y
    }

    /// Project the object onto a canvas as a single pixel.
    /// Perspective: screen = (x / z) * scale, centered.
    pub fn project(&self, canvas: &mut PixelCanvas, r: f32, g: f32, b: f32) {
        let cx = canvas.width() as f32 / 2.0;
        let cy = canvas.height() as f32 / 2.0;
        let sx = cx + (self.x / self.z) * self.scale * 10.0;
        let sy = cy - (self.y / self.z) * self.scale * 10.0;
        let px = sx as u32;
        let py = sy as u32;
        let _ = canvas.set(px, py, r, g, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let o = Object3D::new("box").unwrap();
        assert_eq!(o.scale(), 1.0);
    }

    #[test]
    fn empty_name() {
        assert!(Object3D::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut o = Object3D::new("box").unwrap();
        o.set_position(0.0, 0.0, 1.0);
        o.set_scale(1.0);
        let mut c = PixelCanvas::new(16, 16).unwrap();
        o.project(&mut c, 1.0, 0.0, 0.0);
        let (r, _, _) = c.get(8, 8).unwrap();
        assert_eq!(r, 1.0);
    }
}
