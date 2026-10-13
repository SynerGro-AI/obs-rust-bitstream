//! Safe pure AudioEffect (audio processing core).
//!
//! An effect owns a name and a gain multiplier. Empty names
//! are rejected. No unwraps.
//!
//! Differential: apply a 0.5 effect to a 1.0 channel yields 0.5.

use crate::audio_channel::AudioChannel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct AudioEffect {
    name: String,
    gain: f32,
}

impl AudioEffect {
    pub fn new(name: &str) -> Result<Self, EffectError> {
        if name.is_empty() {
            return Err(EffectError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            gain: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_gain(&mut self, g: f32) {
        self.gain = g.max(0.0);
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    /// Apply the effect gain to a channel's effective gain.
    pub fn apply(&self, channel: &AudioChannel) -> f32 {
        channel.effective_gain() * self.gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let e = AudioEffect::new("compressor").unwrap();
        assert_eq!(e.name(), "compressor");
        assert_eq!(e.gain(), 1.0);
    }

    #[test]
    fn empty_name() {
        assert!(AudioEffect::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut e = AudioEffect::new("pad").unwrap();
        e.set_gain(0.5);
        let c = AudioChannel::new("mic").unwrap();
        assert!((e.apply(&c) - 0.5).abs() < 0.001);
    }
}
