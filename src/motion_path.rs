//! Safe pure MotionPath (path follow core).
//!
//! Holds a list of points. Sample returns the point
//! at a progress along the path. Empty rejected.
//! No unwraps.
//!
//! Differential: sample at 0 is the first point.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    Empty,
}

#[derive(Clone, Debug, Default)]
pub struct MotionPath {
    points: Vec<PathPoint>,
}

impl MotionPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: f32, y: f32, z: f32) {
        self.points.push(PathPoint { x, y, z });
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// progress is 0.0..=1.0.
    pub fn sample(&self, progress: f32) -> Result<PathPoint, PathError> {
        if self.points.is_empty() {
            return Err(PathError::Empty);
        }
        let t = progress.clamp(0.0, 1.0);
        let i = ((self.points.len() - 1) as f32 * t) as usize;
        Ok(self.points[i])
    }

    /// Read a point from live weights.
    pub fn from_weights(&mut self, index: usize, weights: &LiveWeights) {
        if index >= self.points.len() {
            return;
        }
        let nx = format!("path.{}.x", index);
        if let Ok(x) = weights.get(&nx) {
            self.points[index].x = x;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let p = MotionPath::new();
        assert!(p.sample(0.5).is_err());
    }

    #[test]
    fn differential() {
        let mut p = MotionPath::new();
        p.add(0.0, 0.0, 0.0);
        p.add(10.0, 0.0, 0.0);
        assert_eq!(p.sample(0.0).unwrap().x, 0.0);
        assert_eq!(p.sample(1.0).unwrap().x, 10.0);
        let mut w = LiveWeights::new();
        w.set("path.0.x", 5.0).unwrap();
        p.from_weights(0, &w);
        assert_eq!(p.sample(0.0).unwrap().x, 5.0);
    }
}
