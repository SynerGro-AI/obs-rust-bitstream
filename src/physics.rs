//! Safe pure PhysicsSettings (character / scene physics core).
//!
//! Owns gravity, friction, bounce, and mass. AI assist can
//! suggest values and write them. No unwraps.
//!
//! Differential: set gravity + get round-trips; AI assist
//! writes a fallback when the reply has no floats.

use crate::ai::AiAssistant;
use crate::obs_data::ObsData;

#[derive(Clone, Debug)]
pub struct PhysicsSettings {
    gravity: f32,
    friction: f32,
    bounce: f32,
    mass: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            friction: 0.5,
            bounce: 0.2,
            mass: 1.0,
        }
    }
}

impl PhysicsSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }
    pub fn set_gravity(&mut self, v: f32) {
        self.gravity = v;
    }
    pub fn friction(&self) -> f32 {
        self.friction
    }
    pub fn set_friction(&mut self, v: f32) {
        self.friction = v.clamp(0.0, 1.0);
    }
    pub fn bounce(&self) -> f32 {
        self.bounce
    }
    pub fn set_bounce(&mut self, v: f32) {
        self.bounce = v.clamp(0.0, 1.0);
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn set_mass(&mut self, v: f32) {
        self.mass = v.max(0.0);
    }

    /// Ask the assistant for physics values. Parses four floats
    /// (gravity, friction, bounce, mass) if present, otherwise
    /// keeps the current values. Returns true if AI values were applied.
    pub fn assist(&mut self, ai: &AiAssistant, prompt: &str) -> bool {
        let reply = ai.assist(prompt);
        if let Some((g, f, b, m)) = parse_four(&reply) {
            self.set_gravity(g);
            self.set_friction(f);
            self.set_bounce(b);
            self.set_mass(m);
            true
        } else {
            false
        }
    }

    /// Write the current physics into an ObsData bag so other
    /// settings stores can carry them.
    pub fn write_to(&self, data: &mut ObsData) {
        let _ = data.set_double("physics.gravity", self.gravity as f64);
        let _ = data.set_double("physics.friction", self.friction as f64);
        let _ = data.set_double("physics.bounce", self.bounce as f64);
        let _ = data.set_double("physics.mass", self.mass as f64);
    }
}

fn parse_four(s: &str) -> Option<(f32, f32, f32, f32)> {
    let parts: Vec<f32> = s
        .split([',', ' ', '\n'])
        .filter_map(|p| p.trim().parse().ok())
        .collect();
    if parts.len() >= 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = PhysicsSettings::new();
        assert!((p.gravity() + 9.81).abs() < 0.001);
        assert_eq!(p.mass(), 1.0);
    }

    #[test]
    fn clamp() {
        let mut p = PhysicsSettings::new();
        p.set_friction(2.0);
        p.set_mass(-1.0);
        assert_eq!(p.friction(), 1.0);
        assert_eq!(p.mass(), 0.0);
    }

    #[test]
    fn differential() {
        let mut p = PhysicsSettings::new();
        p.set_gravity(-10.0);
        let mut data = ObsData::new();
        p.write_to(&mut data);
        assert_eq!(data.get_double("physics.gravity").unwrap(), -10.0);
    }

    #[test]
    fn ai_assist_keeps_current_on_no_floats() {
        let mut p = PhysicsSettings::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let applied = p.assist(&ai, "suggest physics");
        assert!(!applied);
        assert!((p.gravity() + 9.81).abs() < 0.001);
    }
}
