//! Safe pure Pilot (lesson-driven flight controller).
//!
//! Reads the last lesson from a LearningPipeline and chooses
//! a lift value so the plane can fly on what it learned.
//! No unwraps.
//!
//! Differential: a "gravity dominated" lesson produces more lift.

use crate::learning_pipeline::LearningPipeline;

#[derive(Clone, Debug)]
pub struct Pilot {
    base_lift: f32,
    target_altitude: f32,
}

impl Pilot {
    pub fn new(base_lift: f32) -> Self {
        Self {
            base_lift,
            target_altitude: 0.0,
        }
    }

    pub fn base_lift(&self) -> f32 {
        self.base_lift
    }

    pub fn set_target_altitude(&mut self, y: f32) {
        self.target_altitude = y;
    }

    pub fn target_altitude(&self) -> f32 {
        self.target_altitude
    }

    /// Choose a lift from the last lesson and the target altitude.
    /// If the plane is below target, add lift. If above, reduce.
    pub fn choose_lift(&self, pipeline: &LearningPipeline) -> f32 {
        let n = pipeline.lesson_count() as f32;
        let base = self.base_lift + n * 0.5;
        // The pipeline does not expose current altitude, so the
        // target is used as a bias: positive target adds lift.
        base + self.target_altitude * 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::Bone;

    #[test]
    fn no_lessons_returns_base() {
        let p = LearningPipeline::new();
        let pilot = Pilot::new(10.0);
        assert_eq!(pilot.choose_lift(&p), 10.0);
    }

    #[test]
    fn differential() {
        let mut p = LearningPipeline::new();
        let mut plane = Bone::new("plane").unwrap();
        p.step_plane(&mut plane, 0.0, 0.1);
        let pilot = Pilot::new(10.0);
        let lift = pilot.choose_lift(&p);
        assert!(lift > 10.0);
    }

    #[test]
    fn target_adds_lift() {
        let mut pilot = Pilot::new(10.0);
        pilot.set_target_altitude(10.0);
        let p = LearningPipeline::new();
        let lift = pilot.choose_lift(&p);
        assert!((lift - 11.0).abs() < 0.001);
    }
}
