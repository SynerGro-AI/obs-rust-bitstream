//! Safe pure BlendShape (morph target core).
//!
//! Holds named targets as vertex offsets. Blend mixes
//! them by weight. Empty names rejected. No unwraps.
//!
//! Differential: blend of 0.5 is halfway.

use crate::live_weights::LiveWeights;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlendError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct BlendShape {
    base: Vec<(f32, f32, f32)>,
    targets: BTreeMap<String, Vec<(f32, f32, f32)>>,
    weights: BTreeMap<String, f32>,
}

impl BlendShape {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_base(&mut self, verts: Vec<(f32, f32, f32)>) {
        self.base = verts;
    }

    pub fn add_target(
        &mut self,
        name: &str,
        offsets: Vec<(f32, f32, f32)>,
    ) -> Result<(), BlendError> {
        if name.is_empty() {
            return Err(BlendError::EmptyName);
        }
        self.targets.insert(name.to_string(), offsets);
        self.weights.insert(name.to_string(), 0.0);
        Ok(())
    }

    pub fn set_weight(&mut self, name: &str, weight: f32) -> Result<(), BlendError> {
        let w = self.weights.get_mut(name).ok_or(BlendError::Missing)?;
        *w = weight.clamp(0.0, 1.0);
        Ok(())
    }

    /// Blend base with all targets by their weights.
    pub fn blend(&self) -> Vec<(f32, f32, f32)> {
        let mut out = self.base.clone();
        for (name, offsets) in &self.targets {
            let w = self.weights.get(name).copied().unwrap_or(0.0);
            for (i, (ox, oy, oz)) in offsets.iter().enumerate() {
                if i < out.len() {
                    out[i].0 += ox * w;
                    out[i].1 += oy * w;
                    out[i].2 += oz * w;
                }
            }
        }
        out
    }

    /// Read a target weight from live weights.
    pub fn from_weights(&mut self, name: &str, weights: &LiveWeights) -> Result<(), BlendError> {
        let key = format!("blend.{}", name);
        let w = weights.get(&key).map_err(|_| BlendError::Missing)?;
        self.set_weight(name, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut b = BlendShape::new();
        assert!(b.add_target("", vec![]).is_err());
        assert!(b.set_weight("missing", 1.0).is_err());
    }

    #[test]
    fn differential() {
        let mut b = BlendShape::new();
        b.set_base(vec![(0.0, 0.0, 0.0)]);
        b.add_target("smile", vec![(2.0, 0.0, 0.0)]).unwrap();
        b.set_weight("smile", 0.5).unwrap();
        let out = b.blend();
        assert_eq!(out[0].0, 1.0);
        let mut w = LiveWeights::new();
        w.set("blend.smile", 1.0).unwrap();
        b.from_weights("smile", &w).unwrap();
        assert_eq!(b.blend()[0].0, 2.0);
    }
}
