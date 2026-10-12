//! Safe pure AnimationTimeline (character keyframe core).
//!
//! A timeline owns a name, a duration, and a list of keyframes
//! that set a bone's local translation at a time. Empty names
//! are rejected. No unwraps.
//!
//! Differential: add keyframe + sample at time round-trips.

use crate::bone::Bone;
use crate::matrix4::Matrix4;
use crate::vec4::Vec4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimelineError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Keyframe {
    time_secs: f32,
    bone: String,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Debug)]
pub struct AnimationTimeline {
    name: String,
    duration_secs: f32,
    keys: Vec<Keyframe>,
}

impl AnimationTimeline {
    pub fn new(name: &str, duration_secs: f32) -> Result<Self, TimelineError> {
        if name.is_empty() {
            return Err(TimelineError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            duration_secs,
            keys: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    pub fn add_key(&mut self, time_secs: f32, bone: &str, x: f32, y: f32, z: f32) {
        self.keys.push(Keyframe {
            time_secs,
            bone: bone.to_string(),
            x,
            y,
            z,
        });
        self.keys.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Sample the last key at or before `time` for the named bone
    /// and apply it to the bone's local translation.
    pub fn apply(&self, bone: &mut Bone, time: f32) {
        let mut best: Option<&Keyframe> = None;
        for k in &self.keys {
            if k.bone == bone.name() && k.time_secs <= time {
                best = Some(k);
            }
        }
        if let Some(k) = best {
            bone.set_local_translation(k.x, k.y, k.z);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = AnimationTimeline::new("idle", 2.0).unwrap();
        assert_eq!(t.name(), "idle");
        assert_eq!(t.duration_secs(), 2.0);
    }

    #[test]
    fn empty_name() {
        assert!(AnimationTimeline::new("", 1.0).is_err());
    }

    #[test]
    fn differential() {
        let mut timeline = AnimationTimeline::new("wave", 1.0).unwrap();
        timeline.add_key(0.0, "spine", 0.0, 0.0, 0.0);
        timeline.add_key(0.5, "spine", 0.0, 1.0, 0.0);
        let mut bone = Bone::new("spine").unwrap();
        timeline.apply(&mut bone, 0.6);
        assert_eq!(bone.local().t.y, 1.0);
        assert_eq!(timeline.key_count(), 2);
    }
}
