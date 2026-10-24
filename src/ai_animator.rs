//! Safe pure AiAnimator (AI-driven animation core).
//!
//! Asks an AiAssistant for a sequence of keyframes and applies
//! them to a skeleton over time. Falls back to a simple rise
//! if the reply has no floats. No unwraps.
//!
//! Differential: generate + sample at midpoint round-trips.

use crate::ai::AiAssistant;
use crate::animation_timeline::AnimationTimeline;
use crate::bone::Bone;
use crate::skeleton_connector::SkeletonConnector;

#[derive(Clone, Debug)]
pub struct AiAnimator {
    timeline: AnimationTimeline,
}

impl AiAnimator {
    pub fn new(
        name: &str,
        duration_secs: f32,
    ) -> Result<Self, crate::animation_timeline::TimelineError> {
        Ok(Self {
            timeline: AnimationTimeline::new(name, duration_secs)?,
        })
    }

    pub fn timeline(&self) -> &AnimationTimeline {
        &self.timeline
    }

    /// Ask the assistant for keyframes and bake them onto the
    /// named bone. Falls back to a linear rise from 0 to 1 if
    /// the reply has no usable floats.
    pub fn generate(&mut self, bone_name: &str, ai: &AiAssistant, prompt: &str) {
        let reply = ai.assist(prompt);
        let keys = parse_keys(&reply);
        if keys.is_empty() {
            self.timeline.add_key(0.0, bone_name, 0.0, 0.0, 0.0);
            self.timeline
                .add_key(self.timeline.duration_secs(), bone_name, 0.0, 1.0, 0.0);
        } else {
            for (t, x, y, z) in keys {
                self.timeline.add_key(t, bone_name, x, y, z);
            }
        }
    }

    /// Apply the generated timeline to a bone at a time.
    pub fn apply(&self, bone: &mut Bone, time: f32) {
        self.timeline.apply(bone, time);
    }

    /// Apply the generated timeline to the root of a connector.
    pub fn apply_to_connector(&self, connector: &mut SkeletonConnector, time: f32) {
        self.timeline.apply(connector.root_mut(), time);
    }
}

fn parse_keys(s: &str) -> Vec<(f32, f32, f32, f32)> {
    let mut out = Vec::new();
    for line in s.lines() {
        let parts: Vec<f32> = line
            .split([',', ' '])
            .filter_map(|p| p.trim().parse().ok())
            .collect();
        if parts.len() >= 4 {
            out.push((parts[0], parts[1], parts[2], parts[3]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_fallback() {
        let mut anim = AiAnimator::new("idle", 1.0).unwrap();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        anim.generate("spine", &ai, "suggest animation");
        assert!(anim.timeline().key_count() >= 2);
    }

    #[test]
    fn differential() {
        let mut anim = AiAnimator::new("rise", 1.0).unwrap();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        anim.generate("spine", &ai, "suggest animation");
        let mut bone = Bone::new("spine").unwrap();
        anim.apply(&mut bone, 0.5);
        assert!((bone.local().t.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn apply_to_connector() {
        let mut anim = AiAnimator::new("rise", 1.0).unwrap();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        anim.generate("root", &ai, "suggest animation");
        let mut c = SkeletonConnector::new("root").unwrap();
        anim.apply_to_connector(&mut c, 1.0);
        assert!((c.root().local().t.y - 1.0).abs() < 0.001);
    }
}
