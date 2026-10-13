//! Safe pure AudioBus (audio mixer core).
//!
//! Holds named channels and sums their effective gains into a
//! mix level. Empty and duplicate names are rejected. No unwraps.
//!
//! Differential: two channels at 0.5 sum to 1.0.

use std::collections::HashMap;

use crate::audio_channel::{AudioChannel, ChannelError};
use crate::effect_chain::EffectChain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BusError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<ChannelError> for BusError {
    fn from(_: ChannelError) -> Self {
        BusError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct AudioBus {
    channels: HashMap<String, AudioChannel>,
}

impl AudioBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), BusError> {
        if name.is_empty() {
            return Err(BusError::EmptyName);
        }
        if self.channels.contains_key(name) {
            return Err(BusError::Duplicate);
        }
        let c = AudioChannel::new(name)?;
        self.channels.insert(name.to_string(), c);
        Ok(())
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut AudioChannel, BusError> {
        self.channels.get_mut(name).ok_or(BusError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.channels.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// Sum of every channel's effective gain.
    pub fn mix_level(&self) -> f32 {
        self.channels.values().map(|c| c.effective_gain()).sum()
    }

    /// Apply an effect chain to the mix level.
    pub fn mix_through(&self, chain: &EffectChain) -> f32 {
        let mut g = self.mix_level();
        // EffectChain applies to a channel; we scale the mix by
        // the chain's combined gain via a temporary channel.
        if let Ok(tmp) = AudioChannel::new("mix") {
            let mut tmp = tmp;
            tmp.set_gain(g);
            g = chain.apply(&tmp);
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut b = AudioBus::new();
        b.add("mic").unwrap();
        b.add("music").unwrap();
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut b = AudioBus::new();
        b.add("a").unwrap();
        assert!(matches!(b.add("a"), Err(BusError::Duplicate)));
        assert!(matches!(b.add(""), Err(BusError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut b = AudioBus::new();
        b.add("mic").unwrap();
        b.add("music").unwrap();
        b.get_mut("mic").unwrap().set_gain(0.5);
        b.get_mut("music").unwrap().set_gain(0.5);
        assert!((b.mix_level() - 1.0).abs() < 0.001);
    }

    #[test]
    fn mix_through_chain() {
        let mut b = AudioBus::new();
        b.add("mic").unwrap();
        let mut chain = EffectChain::new();
        chain.add("pad").unwrap();
        chain.get_mut(0).unwrap().set_gain(0.5);
        assert!((b.mix_through(&chain) - 0.5).abs() < 0.001);
    }
}
