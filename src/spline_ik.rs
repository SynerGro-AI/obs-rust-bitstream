//! Safe pure SplineIk (chain follows a curve core).
//!
//! Places joints along a motion path by arc progress.
//! No unwraps.
//!
//! Differential: three joints land on the path.

use crate::live_weights::LiveWeights;
use crate::motion_path::MotionPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplineError {
    Empty,
}

pub struct SplineIk;

impl SplineIk {
    /// Place `count` joints evenly along the path.
    /// Returns (x, y) for each joint.
    pub fn solve(path: &MotionPath, count: usize) -> Result<Vec<(f32, f32)>, SplineError> {
        if path.len() == 0 || count == 0 {
            return Err(SplineError::Empty);
        }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let t = if count == 1 {
                0.0
            } else {
                i as f32 / (count - 1) as f32
            };
            let p = path.sample(t).map_err(|_| SplineError::Empty)?;
            out.push((p.x, p.y));
        }
        Ok(out)
    }

    /// Read joint count from live weights.
    pub fn from_weights(
        path: &MotionPath,
        weights: &LiveWeights,
    ) -> Result<Vec<(f32, f32)>, SplineError> {
        let n = weights.get("spline.count").unwrap_or(3.0) as usize;
        Self::solve(path, n.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let path = MotionPath::new();
        assert!(SplineIk::solve(&path, 3).is_err());
    }

    #[test]
    fn differential() {
        let mut path = MotionPath::new();
        path.add(0.0, 0.0, 0.0);
        path.add(10.0, 0.0, 0.0);
        let joints = SplineIk::solve(&path, 3).unwrap();
        assert_eq!(joints.len(), 3);
        assert_eq!(joints[0].0, 0.0);
        assert_eq!(joints[2].0, 10.0);
        let mut w = LiveWeights::new();
        w.set("spline.count", 5.0).unwrap();
        let j2 = SplineIk::from_weights(&path, &w).unwrap();
        assert_eq!(j2.len(), 5);
    }
}
