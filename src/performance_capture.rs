//! Safe pure PerformanceCapture (actor recording core).
//!
//! Records bone transforms over time into an AnimationTimeline
//! so a real or digital actor can drive the skeleton. Empty
//! names are rejected. No unwraps.
//!
//! Differential: record a sample + apply at that time round-trips.

use crate::animation_timeline::AnimationTimeline;
use crate::bone::Bone;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Sample {
    time_secs: f32,
    bone: String,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Debug)]
pub struct PerformanceCapture {
    name: String,
    samples: Vec<Sample>,
}

impl PerformanceCapture {
    pub fn new(name: &str) -> Result<Self, CaptureError> {
        if name.is_empty() {
            return Err(CaptureError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            samples: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a bone's current local translation at a time.
    pub fn record(&mut self, time_secs: f32, bone: &Bone) {
        let t = bone.local().t;
        self.samples.push(Sample {
            time_secs,
            bone: bone.name().to_string(),
            x: t.x,
            y: t.y,
            z: t.z,
        });
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Bake the recorded samples into an animation timeline.
    pub fn bake(&self, timeline: &mut AnimationTimeline) {
        for s in &self.samples {
            timeline.add_key(s.time_secs, &s.bone, s.x, s.y, s.z);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = PerformanceCapture::new("take1").unwrap();
        assert_eq!(c.name(), "take1");
        assert_eq!(c.sample_count(), 0);
    }

    #[test]
    fn empty_name() {
        assert!(PerformanceCapture::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut bone = Bone::new("spine").unwrap();
        bone.set_local_translation(1.0, 2.0, 3.0);
        let mut cap = PerformanceCapture::new("take1").unwrap();
        cap.record(0.5, &bone);
        let mut timeline = AnimationTimeline::new("baked", 1.0).unwrap();
        cap.bake(&mut timeline);
        let mut out = Bone::new("spine").unwrap();
        timeline.apply(&mut out, 0.5);
        assert_eq!(out.local().t.x, 1.0);
        assert_eq!(out.local().t.y, 2.0);
        assert_eq!(cap.sample_count(), 1);
    }
}
