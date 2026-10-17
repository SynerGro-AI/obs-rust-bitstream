//! Safe pure Layer3D (layer with depth core).
//!
//! A 3D layer owns a name, a position, and a depth.
//! Empty names rejected. No unwraps.
//!
//! Differential: set position + get round-trips.

use crate::live_weights::LiveWeights;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layer3DError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Layer3D {
    name: String,
    x: f32,
    y: f32,
    z: f32,
}

impl Layer3D {
    pub fn new(name: &str) -> Result<Self, Layer3DError> {
        if name.is_empty() {
            return Err(Layer3DError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn position(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    /// Read position from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        let nx = format!("layer3d.{}.x", self.name);
        let ny = format!("layer3d.{}.y", self.name);
        let nz = format!("layer3d.{}.z", self.name);
        if let Ok(x) = weights.get(&nx) {
            self.x = x;
        }
        if let Ok(y) = weights.get(&ny) {
            self.y = y;
        }
        if let Ok(z) = weights.get(&nz) {
            self.z = z;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Layer3DSet {
    layers: BTreeMap<String, Layer3D>,
}

impl Layer3DSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, layer: Layer3D) {
        self.layers.insert(layer.name().to_string(), layer);
    }

    pub fn get(&self, name: &str) -> Result<&Layer3D, Layer3DError> {
        self.layers.get(name).ok_or(Layer3DError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Layer3D, Layer3DError> {
        self.layers.get_mut(name).ok_or(Layer3DError::Missing)
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(Layer3D::new("").is_err());
        let set = Layer3DSet::new();
        assert!(set.get("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut set = Layer3DSet::new();
        let mut l = Layer3D::new("card").unwrap();
        let mut w = LiveWeights::new();
        w.set("layer3d.card.x", 1.0).unwrap();
        w.set("layer3d.card.z", 5.0).unwrap();
        l.from_weights(&w);
        set.add(l);
        assert_eq!(set.get("card").unwrap().position(), (1.0, 0.0, 5.0));
    }
}
