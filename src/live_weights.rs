//! Safe pure LiveWeights (runtime parameter store).
//!
//! Holds named float weights that can change between frames.
//! Empty names rejected. No unwraps.
//!
//! Differential: set + get + update round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeightError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct LiveWeights {
    values: BTreeMap<String, f32>,
}

impl LiveWeights {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: &str, value: f32) -> Result<(), WeightError> {
        if name.is_empty() {
            return Err(WeightError::EmptyName);
        }
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<f32, WeightError> {
        self.values.get(name).copied().ok_or(WeightError::Missing)
    }

    /// Add delta to an existing weight. Missing is an error.
    pub fn update(&mut self, name: &str, delta: f32) -> Result<f32, WeightError> {
        let v = self.get(name)?;
        let next = v + delta;
        self.values.insert(name.to_string(), next);
        Ok(next)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.values.remove(name).is_some()
    }

    pub fn names(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut w = LiveWeights::new();
        assert!(w.set("", 1.0).is_err());
        assert!(w.get("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut w = LiveWeights::new();
        w.set("glow", 0.5).unwrap();
        assert_eq!(w.get("glow").unwrap(), 0.5);
        let next = w.update("glow", 0.2).unwrap();
        assert_eq!(next, 0.7);
        assert!(w.remove("glow"));
        assert!(w.is_empty());
    }
}
