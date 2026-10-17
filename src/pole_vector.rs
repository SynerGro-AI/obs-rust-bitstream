//! Safe pure PoleVector (elbow/knee aim core).
//!
//! Given a chain start, end, and a pole, returns the
//! side the mid joint should bend toward. No unwraps.
//!
//! Differential: a pole to the left bends left.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pole {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct PoleVector {
    pole: Pole,
}

impl PoleVector {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pole: Pole { x, y },
        }
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.pole = Pole { x, y };
    }

    pub fn pole(&self) -> Pole {
        self.pole
    }

    /// Which side of the start-end line the pole is on.
    /// Positive is left, negative is right.
    pub fn side(&self, start: (f32, f32), end: (f32, f32)) -> f32 {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let px = self.pole.x - start.0;
        let py = self.pole.y - start.1;
        dx * py - dy * px
    }

    /// Read pole position from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(x) = weights.get("pole.x") {
            self.pole.x = x;
        }
        if let Ok(y) = weights.get("pole.y") {
            self.pole.y = y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut p = PoleVector::new(0.0, 1.0);
        let left = p.side((0.0, 0.0), (1.0, 0.0));
        assert!(left > 0.0);
        p.set(0.0, -1.0);
        let right = p.side((0.0, 0.0), (1.0, 0.0));
        assert!(right < 0.0);
        let mut w = LiveWeights::new();
        w.set("pole.x", 2.0).unwrap();
        w.set("pole.y", 3.0).unwrap();
        p.from_weights(&w);
        assert_eq!(p.pole().x, 2.0);
        assert_eq!(p.pole().y, 3.0);
    }
}
