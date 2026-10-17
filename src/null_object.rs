//! Safe pure Null (empty transform holder core).
//!
//! A null owns a name and a position. It can be a parent
//! without drawing. Empty names rejected. No unwraps.
//!
//! Differential: set position + get round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NullError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Null {
    name: String,
    x: f32,
    y: f32,
    z: f32,
}

impl Null {
    pub fn new(name: &str) -> Result<Self, NullError> {
        if name.is_empty() {
            return Err(NullError::EmptyName);
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
}

#[derive(Clone, Debug, Default)]
pub struct NullSet {
    nulls: BTreeMap<String, Null>,
}

impl NullSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, null: Null) {
        self.nulls.insert(null.name().to_string(), null);
    }

    pub fn get(&self, name: &str) -> Result<&Null, NullError> {
        self.nulls.get(name).ok_or(NullError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Null, NullError> {
        self.nulls.get_mut(name).ok_or(NullError::Missing)
    }

    pub fn len(&self) -> usize {
        self.nulls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(Null::new("").is_err());
        let set = NullSet::new();
        assert!(set.get("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut set = NullSet::new();
        let mut n = Null::new("root").unwrap();
        n.set_position(1.0, 2.0, 3.0);
        set.add(n);
        assert_eq!(set.get("root").unwrap().position(), (1.0, 2.0, 3.0));
    }
}
