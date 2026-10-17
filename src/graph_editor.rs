//! Safe pure GraphEditor (curve edit core).
//!
//! Holds named curves of (time, value) points.
//! Can insert, move, and sample. Empty names rejected.
//! No unwraps.
//!
//! Differential: insert + sample round-trips.

use crate::live_weights::LiveWeights;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct GraphEditor {
    curves: BTreeMap<String, Vec<(u64, f32)>>,
}

impl GraphEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_curve(&mut self, name: &str) -> Result<(), GraphError> {
        if name.is_empty() {
            return Err(GraphError::EmptyName);
        }
        self.curves.entry(name.to_string()).or_default();
        Ok(())
    }

    pub fn insert(&mut self, name: &str, time: u64, value: f32) -> Result<(), GraphError> {
        let curve = self.curves.get_mut(name).ok_or(GraphError::Missing)?;
        curve.push((time, value));
        curve.sort_by_key(|p| p.0);
        Ok(())
    }

    pub fn move_point(
        &mut self,
        name: &str,
        index: usize,
        time: u64,
        value: f32,
    ) -> Result<(), GraphError> {
        let curve = self.curves.get_mut(name).ok_or(GraphError::Missing)?;
        if index >= curve.len() {
            return Err(GraphError::Missing);
        }
        curve[index] = (time, value);
        curve.sort_by_key(|p| p.0);
        Ok(())
    }

    /// Sample the last point at or before time.
    pub fn sample(&self, name: &str, time: u64) -> Result<f32, GraphError> {
        let curve = self.curves.get(name).ok_or(GraphError::Missing)?;
        if curve.is_empty() {
            return Err(GraphError::Missing);
        }
        let mut out = curve[0].1;
        for (t, v) in curve {
            if *t <= time {
                out = *v;
            } else {
                break;
            }
        }
        Ok(out)
    }

    /// Read a point value from live weights.
    pub fn from_weights(
        &mut self,
        name: &str,
        index: usize,
        weights: &LiveWeights,
    ) -> Result<(), GraphError> {
        let key = format!("graph.{}.{}", name, index);
        let v = weights.get(&key).map_err(|_| GraphError::Missing)?;
        let curve = self.curves.get_mut(name).ok_or(GraphError::Missing)?;
        if index >= curve.len() {
            return Err(GraphError::Missing);
        }
        curve[index].1 = v;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut g = GraphEditor::new();
        assert!(g.add_curve("").is_err());
        assert!(g.sample("missing", 0).is_err());
    }

    #[test]
    fn differential() {
        let mut g = GraphEditor::new();
        g.add_curve("opacity").unwrap();
        g.insert("opacity", 0, 0.0).unwrap();
        g.insert("opacity", 10, 1.0).unwrap();
        assert_eq!(g.sample("opacity", 10).unwrap(), 1.0);
        let mut w = LiveWeights::new();
        w.set("graph.opacity.1", 0.5).unwrap();
        g.from_weights("opacity", 1, &w).unwrap();
        assert_eq!(g.sample("opacity", 10).unwrap(), 0.5);
    }
}
