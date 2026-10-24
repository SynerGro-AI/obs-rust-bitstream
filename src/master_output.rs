//! Safe pure MasterOutput (final audio stage).
//!
//! Takes a bus and an effect chain and produces the final
//! output level. No unwraps.
//!
//! Differential: a 1.0 mix through a 0.5 chain yields 0.5.

use crate::audio_bus::AudioBus;
use crate::effect_chain::EffectChain;

#[derive(Clone, Debug)]
pub struct MasterOutput {
    ceiling: f32,
}

impl MasterOutput {
    pub fn new() -> Self {
        Self { ceiling: 1.0 }
    }

    pub fn set_ceiling(&mut self, c: f32) {
        self.ceiling = c.max(0.0);
    }

    pub fn ceiling(&self) -> f32 {
        self.ceiling
    }

    /// Final level: mix through the chain, clamped to the ceiling.
    pub fn level(&self, bus: &AudioBus, chain: &EffectChain) -> f32 {
        bus.mix_through(chain).min(self.ceiling)
    }
}

impl Default for MasterOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = MasterOutput::new();
        assert_eq!(m.ceiling(), 1.0);
    }

    #[test]
    fn differential() {
        let mut bus = AudioBus::new();
        bus.add("mic").unwrap();
        let mut chain = EffectChain::new();
        chain.add("pad").unwrap();
        chain.get_mut(0).unwrap().set_gain(0.5);
        let master = MasterOutput::new();
        assert!((master.level(&bus, &chain) - 0.5).abs() < 0.001);
    }

    #[test]
    fn ceiling_clamps() {
        let mut bus = AudioBus::new();
        bus.add("a").unwrap();
        bus.add("b").unwrap();
        let chain = EffectChain::new();
        let mut master = MasterOutput::new();
        master.set_ceiling(1.0);
        assert!(master.level(&bus, &chain) <= 1.0);
    }
}
