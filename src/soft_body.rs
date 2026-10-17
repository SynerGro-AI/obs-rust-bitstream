//! Safe pure SoftBody (spring-connected points core).
//!
//! Points are linked by springs. Step pulls neighbors
//! toward rest length. No unwraps.
//!
//! Differential: a stretched spring contracts.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoftPoint {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Clone, Debug)]
pub struct SoftBody {
    points: Vec<SoftPoint>,
    rest: f32,
    stiffness: f32,
}

impl SoftBody {
    pub fn new(rest: f32, stiffness: f32) -> Self {
        Self {
            points: Vec::new(),
            rest,
            stiffness,
        }
    }

    pub fn add(&mut self, x: f32, y: f32) {
        self.points.push(SoftPoint {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
        });
    }

    pub fn points(&self) -> &[SoftPoint] {
        &self.points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// One spring step between consecutive points.
    pub fn step(&mut self, dt: f32) {
        if self.points.len() < 2 {
            return;
        }
        let n = self.points.len();
        let mut forces = vec![(0.0f32, 0.0f32); n];
        for i in 0..n - 1 {
            let a = self.points[i];
            let b = self.points[i + 1];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            let stretch = dist - self.rest;
            let f = stretch * self.stiffness;
            let fx = (dx / dist) * f;
            let fy = (dy / dist) * f;
            forces[i].0 += fx;
            forces[i].1 += fy;
            forces[i + 1].0 -= fx;
            forces[i + 1].1 -= fy;
        }
        for (p, (fx, fy)) in self.points.iter_mut().zip(forces.iter()) {
            p.vx += fx * dt;
            p.vy += fy * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
    }

    /// Read stiffness from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(s) = weights.get("soft.stiffness") {
            self.stiffness = s.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut s = SoftBody::new(1.0, 10.0);
        s.add(0.0, 0.0);
        s.add(3.0, 0.0);
        s.step(0.1);
        let dist = s.points()[1].x - s.points()[0].x;
        assert!(dist < 3.0);
        let mut w = LiveWeights::new();
        w.set("soft.stiffness", 0.0).unwrap();
        s.from_weights(&w);
        assert_eq!(s.stiffness, 0.0);
    }
}
