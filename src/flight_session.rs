//! Safe pure FlightSession (closed-loop flight core).
//!
//! Runs an environment-governed plane under a pilot for a fixed
//! number of steps and records the altitude trace. No unwraps.
//!
//! Differential: a pilot that increases lift after a fall
//! produces a rising altitude by the end of the session.

use crate::bone::Bone;
use crate::environment::Environment;
use crate::learning_pipeline::LearningPipeline;
use crate::pilot::Pilot;
use crate::wind::Wind;

#[derive(Clone, Debug)]
pub struct FlightSession {
    altitudes: Vec<f32>,
}

impl FlightSession {
    pub fn new() -> Self {
        Self {
            altitudes: Vec::new(),
        }
    }

    pub fn altitudes(&self) -> &[f32] {
        &self.altitudes
    }

    pub fn len(&self) -> usize {
        self.altitudes.len()
    }

    /// Run the closed loop for `steps` frames.
    /// Environment governs, pilot chooses lift, pipeline steps.
    pub fn run(
        &mut self,
        env: &Environment,
        pilot: &Pilot,
        plane: &mut Bone,
        steps: usize,
        dt: f32,
    ) {
        let mut pipeline = LearningPipeline::new();
        pipeline.govern(env);
        for _ in 0..steps {
            let lift = pilot.choose_lift(&pipeline);
            pipeline.step_plane(plane, lift, dt);
            Wind::apply(env, plane, dt);
            self.altitudes.push(plane.local().t.y);
        }
    }

    /// True if the final altitude is higher than the first.
    pub fn rose(&self) -> bool {
        match (self.altitudes.first(), self.altitudes.last()) {
            (Some(a), Some(b)) => b > a,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_records() {
        let mut s = FlightSession::new();
        let env = Environment::new("earth").unwrap();
        let pilot = Pilot::new(20.0);
        let mut plane = Bone::new("plane").unwrap();
        s.run(&env, &pilot, &mut plane, 5, 0.1);
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn differential() {
        let mut s = FlightSession::new();
        let env = Environment::new("earth").unwrap();
        let pilot = Pilot::new(15.0);
        let mut plane = Bone::new("plane").unwrap();
        s.run(&env, &pilot, &mut plane, 10, 0.1);
        assert!(s.rose());
    }

    #[test]
    fn wind_moves_horizontal() {
        let mut s = FlightSession::new();
        let mut env = Environment::new("earth").unwrap();
        env.set_wind_x(10.0);
        let pilot = Pilot::new(15.0);
        let mut plane = Bone::new("plane").unwrap();
        s.run(&env, &pilot, &mut plane, 5, 0.1);
        assert!((plane.local().t.x - 5.0).abs() < 0.001);
    }
}
