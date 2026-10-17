//! Safe pure SkinWeights (vertex-to-joint weights core).
//!
//! Maps a vertex to joint influences that sum to 1.0.
//! Empty names rejected. No unwraps.
//!
//! Differential: set weight + normalize round-trips.

use crate::live_weights::LiveWeights;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct SkinWeights {
    influences: BTreeMap<usize, Vec<(String, f32)>>,
}

impl SkinWeights {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an influence. Name must be non-empty.
    pub fn add(&mut self, vertex: usize, joint: &str, weight: f32) -> Result<(), SkinError> {
        if joint.is_empty() {
            return Err(SkinError::EmptyName);
        }
        let list = self.influences.entry(vertex).or_default();
        list.push((joint.to_string(), weight.max(0.0)));
        Ok(())
    }

    /// Normalize influences for a vertex so they sum to 1.0.
    pub fn normalize(&mut self, vertex: usize) -> Result<(), SkinError> {
        let list = self.influences.get_mut(&vertex).ok_or(SkinError::Missing)?;
        let sum: f32 = list.iter().map(|(_, w)| w).sum();
        if sum > 0.0 {
            for (_, w) in list.iter_mut() {
                *w /= sum;
            }
        }
        Ok(())
    }

    pub fn influences(&self, vertex: usize) -> Result<&[(String, f32)], SkinError> {
        self.influences
            .get(&vertex)
            .map(|v| v.as_slice())
            .ok_or(SkinError::Missing)
    }

    /// Read a weight from live weights and set it.
    pub fn from_weights(
        &mut self,
        vertex: usize,
        joint: &str,
        weights: &LiveWeights,
    ) -> Result<(), SkinError> {
        let key = format!("skin.{}.{}", vertex, joint);
        let w = weights.get(&key).map_err(|_| SkinError::Missing)?;
        self.add(vertex, joint, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut s = SkinWeights::new();
        assert!(s.add(0, "", 1.0).is_err());
        assert!(s.influences(0).is_err());
    }

    #[test]
    fn differential() {
        let mut s = SkinWeights::new();
        s.add(0, "arm", 2.0).unwrap();
        s.add(0, "body", 2.0).unwrap();
        s.normalize(0).unwrap();
        let inf = s.influences(0).unwrap();
        assert!((inf[0].1 - 0.5).abs() < 0.01);
        let mut w = LiveWeights::new();
        w.set("skin.1.arm", 0.8).unwrap();
        s.from_weights(1, "arm", &w).unwrap();
        assert_eq!(s.influences(1).unwrap()[0].1, 0.8);
    }
}
