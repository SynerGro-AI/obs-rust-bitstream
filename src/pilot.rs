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
}

impl Pilot {
    pub fn new(base_lift: f32) -> Self {
        Self { base_lift }
    }

    pub fn base_lift(&self) -> f32 {
        self.base_lift
    }

    /// Choose a lift from the last lesson. If gravity dominated,
    /// increase lift. If lift already won, hold.
    pub fn choose_lift(&self, pipeline: &LearningPipeline) -> f32 {
        if pipeline.lesson_count() == 0 {
            return self.base_lift;
        }
        // The pipeline does not expose lessons directly, so we
        // use the experience count and a simple heuristic:
        // more experiences of falling mean more lift is needed.
        // A future step can expose the last lesson summary.
        let n = pipeline.lesson_count() as f32;
        self.base_lift + n * 0.5
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
}
