//! Safe pure PhysicsSim (AI-driven physics core).
//!
//! Steps a bone under gravity, velocity, and damping. An
//! AiAssistant can suggest the step parameters. No unwraps.
//!
//! Differential: step once under gravity moves the bone down.

use crate::ai::AiAssistant;
use crate::bone::Bone;
use crate::physics::PhysicsSettings;

#[derive(Clone, Debug)]
pub struct PhysicsSim {
    velocity_y: f32,
    settings: PhysicsSettings,
}

impl PhysicsSim {
    pub fn new() -> Self {
        Self {
            velocity_y: 0.0,
            settings: PhysicsSettings::new(),
        }
    }

    pub fn settings(&self) -> &PhysicsSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut PhysicsSettings {
        &mut self.settings
    }

    /// Ask the assistant for physics values and apply them.
    pub fn assist(&mut self, ai: &AiAssistant, prompt: &str) -> bool {
        self.settings.assist(ai, prompt)
    }

    /// Step the bone one frame. Gravity accelerates velocity,
    /// damping slows it, position integrates.
    pub fn step(&mut self, bone: &mut Bone, dt: f32) {
        let g = self.settings.gravity();
        let damp = 1.0 - self.settings.friction() * dt;
        self.velocity_y = (self.velocity_y + g * dt) * damp;
        let y = bone.local().t.y + self.velocity_y * dt;
        bone.set_local_translation(bone.local().t.x, y, bone.local().t.z);
    }

    pub fn velocity_y(&self) -> f32 {
        self.velocity_y
    }
}

impl Default for PhysicsSim {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_under_gravity() {
        let mut sim = PhysicsSim::new();
        let mut bone = Bone::new("spine").unwrap();
        sim.step(&mut bone, 0.1);
        assert!(bone.local().t.y < 0.0);
    }

    #[test]
    fn differential() {
        let mut sim = PhysicsSim::new();
        sim.settings_mut().set_gravity(-10.0);
        let mut bone = Bone::new("spine").unwrap();
        sim.step(&mut bone, 0.1);
        assert!((bone.local().t.y + 0.1).abs() < 0.05);
    }

    #[test]
    fn ai_assist_keeps_defaults() {
        let mut sim = PhysicsSim::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let applied = sim.assist(&ai, "suggest physics");
        assert!(!applied);
        assert!((sim.settings().gravity() + 9.81).abs() < 0.001);
    }
}
