//! Safe pure RigidBody (position, velocity, mass core).
//!
//! Steps under gravity and integrates velocity.
//! No unwraps.
//!
//! Differential: one step moves the body down.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub gravity: f32,
}

impl RigidBody {
    pub fn new(mass: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            mass: mass.max(0.001),
            gravity: -9.8,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.vx = vx;
        self.vy = vy;
    }

    /// Advance by dt seconds.
    pub fn step(&mut self, dt: f32) {
        self.vy += self.gravity * dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }

    /// Apply an impulse (changes velocity by impulse / mass).
    pub fn impulse(&mut self, ix: f32, iy: f32) {
        self.vx += ix / self.mass;
        self.vy += iy / self.mass;
    }

    /// Read gravity and mass from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(g) = weights.get("rigid.gravity") {
            self.gravity = g;
        }
        if let Ok(m) = weights.get("rigid.mass") {
            self.mass = m.max(0.001);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut b = RigidBody::new(1.0);
        b.set_position(0.0, 10.0);
        b.step(0.1);
        assert!(b.y < 10.0);
        let mut w = LiveWeights::new();
        w.set("rigid.gravity", 0.0).unwrap();
        w.set("rigid.mass", 2.0).unwrap();
        b.from_weights(&w);
        assert_eq!(b.mass, 2.0);
        b.impulse(2.0, 0.0);
        assert_eq!(b.vx, 1.0);
    }
}
