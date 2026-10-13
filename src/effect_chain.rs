//! Safe pure EffectChain (ordered audio processing core).
//!
//! Holds effects and applies them in order to a channel.
//! Empty names are rejected. No unwraps.
//!
//! Differential: two 0.5 effects on a 1.0 channel yield 0.25.

use crate::audio_channel::AudioChannel;
use crate::audio_effect::{AudioEffect, EffectError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainError {
    EmptyName,
}

impl From<EffectError> for ChainError {
    fn from(_: EffectError) -> Self {
        ChainError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectChain {
    effects: Vec<AudioEffect>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), ChainError> {
        let e = AudioEffect::new(name)?;
        self.effects.push(e);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioEffect> {
        self.effects.get_mut(index)
    }

    /// Apply every effect in order to the channel's effective gain.
    pub fn apply(&self, channel: &AudioChannel) -> f32 {
        let mut g = channel.effective_gain();
        for e in &self.effects {
            g *= e.gain();
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut c = EffectChain::new();
        c.add("a").unwrap();
        c.add("b").unwrap();
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn empty_name() {
        let mut c = EffectChain::new();
        assert!(c.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut chain = EffectChain::new();
        chain.add("a").unwrap();
        chain.add("b").unwrap();
        chain.get_mut(0).unwrap().set_gain(0.5);
        chain.get_mut(1).unwrap().set_gain(0.5);
        let ch = AudioChannel::new("mic").unwrap();
        assert!((chain.apply(&ch) - 0.25).abs() < 0.001);
    }
}
