//! Safe pure AiOperator (news crew core).
//!
//! One assistant acts as camera operator, lights operator,
//! and sound mixer. It frames the camera, sets a light
//! level, and sets a mix level from a topic. No unwraps.
//!
//! Differential: a topic produces a frame, a light, and a mix.

use crate::ai::AiAssistant;
use crate::live_camera::LiveCamera;

#[derive(Clone, Debug)]
pub struct CrewState {
    pub pan: f32,
    pub zoom: f32,
    pub light: f32,
    pub mix: f32,
}

pub struct AiOperator;

impl AiOperator {
    /// Run the crew for a topic. Frames the camera, returns
    /// the light and mix levels.
    pub fn run(ai: &AiAssistant, camera: &mut LiveCamera, topic: &str) -> CrewState {
        let reply = ai.assist(&format!("topic: {topic}")).to_lowercase();
        let zoom = if reply.contains("close") { 3.0 } else { 1.5 };
        let pan = if reply.contains("left") { -0.3 } else { 0.0 };
        let light = if reply.contains("bright") { 1.0 } else { 0.7 };
        let mix = if reply.contains("quiet") { 0.5 } else { 1.0 };
        camera.set_zoom(zoom);
        camera.set_pan(pan);
        CrewState {
            pan,
            zoom,
            light,
            mix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiConfig;

    #[test]
    fn run() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let mut cam = LiveCamera::new("desk").unwrap();
        cam.go_live();
        let state = AiOperator::run(&ai, &mut cam, "weather");
        assert!(cam.is_live());
        assert!(state.zoom >= 1.0);
        assert!(state.light > 0.0);
    }

    #[test]
    fn differential() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let mut cam = LiveCamera::new("desk").unwrap();
        let state = AiOperator::run(&ai, &mut cam, "sports");
        assert!(state.mix > 0.0);
    }
}
