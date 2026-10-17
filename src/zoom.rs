//! Safe pure Zoom (view scale core).
//!
//! Holds a zoom factor and a pan offset. Zoom in
//! multiplies, zoom out divides. No unwraps.
//!
//! Differential: zoom in raises the factor, zoom out lowers it.

#[derive(Clone, Debug, PartialEq)]
pub enum ZoomError {
    TooSmall,
}

#[derive(Clone, Debug)]
pub struct Zoom {
    factor: f32,
    pan_x: f32,
    pan_y: f32,
}

impl Zoom {
    pub fn new() -> Self {
        Self {
            factor: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn factor(&self) -> f32 {
        self.factor
    }

    /// Multiply the factor by 2.
    pub fn zoom_in(&mut self) {
        self.factor = (self.factor * 2.0).min(64.0);
    }

    /// Divide the factor by 2.
    pub fn zoom_out(&mut self) -> Result<(), ZoomError> {
        let next = self.factor / 2.0;
        if next < 0.0625 {
            return Err(ZoomError::TooSmall);
        }
        self.factor = next;
        Ok(())
    }

    pub fn set_pan(&mut self, x: f32, y: f32) {
        self.pan_x = x;
        self.pan_y = y;
    }

    pub fn pan(&self) -> (f32, f32) {
        (self.pan_x, self.pan_y)
    }

    /// Map a view pixel to a frame pixel.
    pub fn to_frame(&self, vx: f32, vy: f32) -> (f32, f32) {
        ((vx - self.pan_x) / self.factor, (vy - self.pan_y) / self.factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_small() {
        let mut z = Zoom::new();
        for _ in 0..10 {
            let _ = z.zoom_out();
        }
        assert!(z.zoom_out().is_err());
    }

    #[test]
    fn differential() {
        let mut z = Zoom::new();
        z.zoom_in();
        assert_eq!(z.factor(), 2.0);
        z.zoom_out().unwrap();
        assert_eq!(z.factor(), 1.0);
        z.set_pan(10.0, 20.0);
        let (fx, fy) = z.to_frame(10.0, 20.0);
        assert_eq!(fx, 0.0);
        assert_eq!(fy, 0.0);
    }
}
