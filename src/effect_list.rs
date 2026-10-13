//! Safe pure EffectList (scene effect registry).
//!
//! Maps a name to an effect. Empty names and duplicates
//! rejected. No unwraps.
//!
//! Differential: add + trigger round-trips.

use crate::effect::{Effect, FxError};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct EffectList {
    effects: BTreeMap<String, Effect>,
}

impl EffectList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), ListError> {
        if name.is_empty() {
            return Err(ListError::EmptyName);
        }
        if self.effects.contains_key(name) {
            return Err(ListError::Duplicate);
        }
        let fx = Effect::new(name).map_err(|e| match e {
            FxError::EmptyName => ListError::EmptyName,
        })?;
        self.effects.insert(name.to_string(), fx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Effect> {
        self.effects.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Effect> {
        self.effects.get_mut(name)
    }

    pub fn trigger(&mut self, name: &str, intensity: f32) -> Result<(), ListError> {
        let fx = self.effects.get_mut(name).ok_or(ListError::Missing)?;
        fx.trigger(intensity);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn names(&self) -> Vec<&str> {
        self.effects.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let l = EffectList::new();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn empty_name() {
        let mut l = EffectList::new();
        assert!(l.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut l = EffectList::new();
        l.add("western").unwrap();
        l.add("sunset").unwrap();
        l.trigger("western", 0.8).unwrap();
        assert!(l.get("western").unwrap().active());
        assert_eq!(l.len(), 2);
    }
}
