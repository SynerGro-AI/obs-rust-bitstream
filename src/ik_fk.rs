//! Safe pure IkFk (inverse / forward kinematics core).
//!
//! FK applies a local rotation. IK solves a two-bone
//! chain toward a target. No unwraps.
//!
//! Differential: FK changes a joint angle, IK reaches.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveMode {
    Fk,
    Ik,
}

#[derive(Clone, Debug)]
pub struct IkFk {
    mode: SolveMode,
    angle: f32,
    length_a: f32,
    length_b: f32,
}

impl IkFk {
    pub fn new(length_a: f32, length_b: f32) -> Self {
        Self {
            mode: SolveMode::Fk,
            angle: 0.0,
            length_a,
            length_b,
        }
    }

    pub fn mode(&self) -> SolveMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: SolveMode) {
        self.mode = mode;
    }

    /// Forward kinematics: set the joint angle.
    pub fn fk(&mut self, angle: f32) {
        self.mode = SolveMode::Fk;
        self.angle = angle;
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Inverse kinematics: solve a two-bone chain toward (tx, ty).
    /// Returns the elbow angle.
    pub fn ik(&mut self, tx: f32, ty: f32) -> f32 {
        self.mode = SolveMode::Ik;
        let dist = (tx * tx + ty * ty).sqrt();
        let max = self.length_a + self.length_b;
        let d = dist.min(max).max((self.length_a - self.length_b).abs());
        let cos = ((d * d - self.length_a * self.length_a - self.length_b * self.length_b)
            / (-2.0 * self.length_a * self.length_b))
            .clamp(-1.0, 1.0);
        self.angle = cos.acos();
        self.angle
    }

    /// Read mode and angle from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(m) = weights.get("ikfk.mode") {
            self.mode = if m > 0.5 { SolveMode::Ik } else { SolveMode::Fk };
        }
        if let Ok(a) = weights.get("ikfk.angle") {
            self.angle = a;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut k = IkFk::new(1.0, 1.0);
        k.fk(0.5);
        assert_eq!(k.angle(), 0.5);
        let a = k.ik(1.0, 0.0);
        assert!(a > 0.0);
        let mut w = LiveWeights::new();
        w.set("ikfk.mode", 0.0).unwrap();
        w.set("ikfk.angle", 1.2).unwrap();
        k.from_weights(&w);
        assert_eq!(k.mode(), SolveMode::Fk);
        assert_eq!(k.angle(), 1.2);
    }
}
