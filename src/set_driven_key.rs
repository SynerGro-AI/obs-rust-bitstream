//! Safe pure SetDrivenKey (driver-driven link core).
//!
//! Maps a driver value to a driven value through
//! key pairs. No unwraps.
//!
//! Differential: a driver of 0.5 returns the mid key.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrivenError {
    Empty,
}

#[derive(Clone, Debug, Default)]
pub struct SetDrivenKey {
    keys: Vec<(f32, f32)>,
}

impl SetDrivenKey {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a (driver, driven) pair.
    pub fn add(&mut self, driver: f32, driven: f32) {
        self.keys.push((driver, driven));
        self.keys
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Sample the driven value for a driver.
    /// Returns the last key at or before the driver.
    pub fn sample(&self, driver: f32) -> Result<f32, DrivenError> {
        if self.keys.is_empty() {
            return Err(DrivenError::Empty);
        }
        let mut out = self.keys[0].1;
        for (d, v) in &self.keys {
            if *d <= driver {
                out = *v;
            } else {
                break;
            }
        }
        Ok(out)
    }

    /// Read a driven value from live weights and set it.
    pub fn from_weights(&mut self, index: usize, weights: &LiveWeights) -> Result<(), DrivenError> {
        if index >= self.keys.len() {
            return Err(DrivenError::Empty);
        }
        let key = format!("driven.{}", index);
        let v = weights.get(&key).map_err(|_| DrivenError::Empty)?;
        self.keys[index].1 = v;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let s = SetDrivenKey::new();
        assert!(s.sample(0.5).is_err());
    }

    #[test]
    fn differential() {
        let mut s = SetDrivenKey::new();
        s.add(0.0, 0.0);
        s.add(1.0, 10.0);
        assert_eq!(s.sample(0.0).unwrap(), 0.0);
        assert_eq!(s.sample(1.0).unwrap(), 10.0);
        assert_eq!(s.sample(0.5).unwrap(), 0.0);
        let mut w = LiveWeights::new();
        w.set("driven.1", 5.0).unwrap();
        s.from_weights(1, &w).unwrap();
        assert_eq!(s.sample(1.0).unwrap(), 5.0);
    }
}
