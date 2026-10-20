//! Safe pure Constraint (aim and parent constraint core).
//!
//! Aim points a joint at a target. Parent copies a
//! parent's position. No unwraps.
//!
//! Differential: aim sets a direction, parent copies.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintKind {
    Aim,
    Parent,
}

#[derive(Clone, Debug)]
pub struct Constraint {
    kind: ConstraintKind,
    weight: f32,
}

impl Constraint {
    pub fn new(kind: ConstraintKind) -> Self {
        Self { kind, weight: 1.0 }
    }

    pub fn kind(&self) -> ConstraintKind {
        self.kind
    }

    pub fn set_weight(&mut self, w: f32) {
        self.weight = w.clamp(0.0, 1.0);
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// Aim: return a direction from source to target, scaled by weight.
    pub fn aim(&self, sx: f32, sy: f32, tx: f32, ty: f32) -> (f32, f32) {
        let dx = (tx - sx) * self.weight;
        let dy = (ty - sy) * self.weight;
        (dx, dy)
    }

    /// Parent: blend source toward parent by weight.
    pub fn parent(&self, sx: f32, sy: f32, px: f32, py: f32) -> (f32, f32) {
        let x = sx + (px - sx) * self.weight;
        let y = sy + (py - sy) * self.weight;
        (x, y)
    }

    /// Read weight from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights, name: &str) {
        if let Ok(w) = weights.get(name) {
            self.set_weight(w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut c = Constraint::new(ConstraintKind::Aim);
        let (dx, _dy) = c.aim(0.0, 0.0, 2.0, 0.0);
        assert_eq!(dx, 2.0);
        let p = Constraint::new(ConstraintKind::Parent);
        let (x, _y) = p.parent(0.0, 0.0, 4.0, 0.0);
        assert_eq!(x, 4.0);
        let mut w = LiveWeights::new();
        w.set("constraint.aim", 0.5).unwrap();
        c.from_weights(&w, "constraint.aim");
        assert_eq!(c.weight(), 0.5);
    }
}
