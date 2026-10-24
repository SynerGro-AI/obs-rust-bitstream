//! Safe pure Particles (simple particle system core).
//!
//! Holds particles with position and velocity. Step
//! advances them by gravity. No unwraps.
//!
//! Differential: one step moves a particle down.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
}

#[derive(Clone, Debug)]
pub struct Particles {
    particles: Vec<Particle>,
    gravity: f32,
}

impl Particles {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            gravity: -9.8,
        }
    }

    pub fn emit(&mut self, x: f32, y: f32, vx: f32, vy: f32) {
        self.particles.push(Particle {
            x,
            y,
            vx,
            vy,
            life: 1.0,
        });
    }

    pub fn set_gravity(&mut self, g: f32) {
        self.gravity = g;
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Advance by dt seconds.
    pub fn step(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.vy += self.gravity * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life = (p.life - dt).max(0.0);
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    /// Read gravity from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(g) = weights.get("particles.gravity") {
            self.gravity = g;
        }
    }
}

impl Default for Particles {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut ps = Particles::new();
        ps.emit(0.0, 10.0, 0.0, 0.0);
        ps.step(0.1);
        assert!(ps.particles()[0].y < 10.0);
        let mut w = LiveWeights::new();
        w.set("particles.gravity", 0.0).unwrap();
        ps.from_weights(&w);
        let y = ps.particles()[0].y;
        ps.step(0.1);
        // gravity 0 so y changes only by existing velocity
        assert!(ps.particles()[0].y <= y);
    }
}
