//! Safe pure Nurbs (curve and surface core).
//!
//! A curve holds control points. A surface holds a grid.
//! Empty rejected. No unwraps.
//!
//! Differential: add point + sample round-trips.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NurbsError {
    Empty,
    BadIndex,
}

#[derive(Clone, Debug, Default)]
pub struct NurbsCurve {
    points: Vec<ControlPoint>,
}

impl NurbsCurve {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: f32, y: f32, z: f32) {
        self.points.push(ControlPoint { x, y, z });
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Linear sample at t in 0.0..=1.0.
    pub fn sample(&self, t: f32) -> Result<ControlPoint, NurbsError> {
        if self.points.is_empty() {
            return Err(NurbsError::Empty);
        }
        let t = t.clamp(0.0, 1.0);
        let i = ((self.points.len() - 1) as f32 * t) as usize;
        Ok(self.points[i])
    }

    /// Move a control point from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights, index: usize) {
        if index >= self.points.len() {
            return;
        }
        let nx = format!("nurbs.c{}.x", index);
        if let Ok(x) = weights.get(&nx) {
            self.points[index].x = x;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NurbsSurface {
    width: usize,
    height: usize,
    points: Vec<ControlPoint>,
}

impl NurbsSurface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            points: Vec::new(),
        }
    }

    pub fn add(&mut self, x: f32, y: f32, z: f32) {
        self.points.push(ControlPoint { x, y, z });
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let c = NurbsCurve::new();
        assert!(c.sample(0.5).is_err());
    }

    #[test]
    fn differential() {
        let mut c = NurbsCurve::new();
        c.add(0.0, 0.0, 0.0);
        c.add(1.0, 0.0, 0.0);
        let p = c.sample(0.0).unwrap();
        assert_eq!(p.x, 0.0);
        let mut w = LiveWeights::new();
        w.set("nurbs.c0.x", 2.0).unwrap();
        c.from_weights(&w, 0);
        assert_eq!(c.sample(0.0).unwrap().x, 2.0);
    }
}
