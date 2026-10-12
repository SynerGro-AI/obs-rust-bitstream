//! Safe pure LearningPipeline (physics-governed AI learning core).
//!
//! Runs a body under real physics (gravity, drag, lift), records
//! each step, and extracts lessons only from the governed
//! outcome. No unwraps.
//!
//! Differential: a plane that applies lift rises; a lesson is
//! recorded only when the physics step moved the body.

use crate::ai::AiAssistant;
use crate::bone::Bone;
use crate::environment::Environment;
use crate::physics::PhysicsSettings;

#[derive(Clone, Debug)]
pub struct Experience {
    time: f32,
    altitude: f32,
    velocity_y: f32,
    lift: f32,
}

#[derive(Clone, Debug)]
pub struct Lesson {
    pub summary: String,
    pub altitude_delta: f32,
}

#[derive(Clone, Debug)]
pub struct LearningPipeline {
    settings: PhysicsSettings,
    experiences: Vec<Experience>,
    lessons: Vec<Lesson>,
    time: f32,
}

impl LearningPipeline {
    pub fn new() -> Self {
        Self {
            settings: PhysicsSettings::new(),
            experiences: Vec::new(),
            lessons: Vec::new(),
            time: 0.0,
        }
    }

    pub fn settings_mut(&mut self) -> &mut PhysicsSettings {
        &mut self.settings
    }

    /// Apply an environment's laws so every subsequent step is
    /// governed by that world.
    pub fn govern(&mut self, env: &Environment) {
        env.govern(&mut self.settings);
    }

    pub fn experience_count(&self) -> usize {
        self.experiences.len()
    }

    pub fn lesson_count(&self) -> usize {
        self.lessons.len()
    }

    /// Step a plane (bone) under gravity + drag + applied lift.
    /// Records the experience. Extracts a lesson only if the
    /// physics step actually moved the body.
    pub fn step_plane(&mut self, plane: &mut Bone, lift: f32, dt: f32) {
        let g = self.settings.gravity();
        let drag = self.settings.friction();
        let before = plane.local().t.y;
        let accel = g + lift - drag * plane.local().t.y;
        let y = before + accel * dt;
        plane.set_local_translation(plane.local().t.x, y, plane.local().t.z);
        self.time += dt;
        self.experiences.push(Experience {
            time: self.time,
            altitude: y,
            velocity_y: accel,
            lift,
        });
        let delta = y - before;
        if delta.abs() > 0.0001 {
            self.lessons.push(Lesson {
                summary: if delta > 0.0 {
                    "lift overcame gravity".to_string()
                } else {
                    "gravity dominated".to_string()
                },
                altitude_delta: delta,
            });
        }
    }

    /// Ask the assistant to review the recorded lessons.
    pub fn review(&self, ai: &AiAssistant) -> String {
        let count = self.lesson_count();
        ai.assist(&format!("review {} physics lessons", count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_rises() {
        let mut p = LearningPipeline::new();
        p.settings_mut().set_gravity(-9.81);
        let mut plane = Bone::new("plane").unwrap();
        p.step_plane(&mut plane, 20.0, 0.1);
        assert!(plane.local().t.y > 0.0);
        assert_eq!(p.lesson_count(), 1);
    }

    #[test]
    fn gravity_falls() {
        let mut p = LearningPipeline::new();
        let mut plane = Bone::new("plane").unwrap();
        p.step_plane(&mut plane, 0.0, 0.1);
        assert!(plane.local().t.y < 0.0);
        assert_eq!(p.lessons[0].summary, "gravity dominated");
    }

    #[test]
    fn differential() {
        let mut p = LearningPipeline::new();
        let mut plane = Bone::new("plane").unwrap();
        p.step_plane(&mut plane, 15.0, 0.1);
        assert_eq!(p.experience_count(), 1);
        assert!(p.lesson_count() >= 1);
    }

    #[test]
    fn review() {
        let p = LearningPipeline::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let r = p.review(&ai);
        assert!(!r.is_empty());
    }

    #[test]
    fn governed_by_environment() {
        let mut p = LearningPipeline::new();
        let mut env = Environment::new("mars").unwrap();
        env.set_gravity(-3.71);
        p.govern(&env);
        let mut plane = Bone::new("plane").unwrap();
        p.step_plane(&mut plane, 0.0, 0.1);
        assert!(plane.local().t.y > -0.5);
        assert!(plane.local().t.y < 0.0);
    }
}
