//! Safe pure LayerGroup (named group of layers core).
//!
//! Holds named groups of layer names. Empty names
//! rejected. No unwraps.
//!
//! Differential: add member + list round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct LayerGroup {
    groups: BTreeMap<String, Vec<String>>,
}

impl LayerGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, name: &str) -> Result<(), GroupError> {
        if name.is_empty() {
            return Err(GroupError::EmptyName);
        }
        self.groups.entry(name.to_string()).or_default();
        Ok(())
    }

    pub fn add_member(&mut self, group: &str, layer: &str) -> Result<(), GroupError> {
        if group.is_empty() || layer.is_empty() {
            return Err(GroupError::EmptyName);
        }
        let g = self.groups.get_mut(group).ok_or(GroupError::Missing)?;
        if !g.contains(&layer.to_string()) {
            g.push(layer.to_string());
        }
        Ok(())
    }

    pub fn members(&self, group: &str) -> Result<&[String], GroupError> {
        self.groups.get(group).map(|v| v.as_slice()).ok_or(GroupError::Missing)
    }

    pub fn remove_member(&mut self, group: &str, layer: &str) -> Result<(), GroupError> {
        let g = self.groups.get_mut(group).ok_or(GroupError::Missing)?;
        g.retain(|l| l != layer);
        Ok(())
    }

    pub fn delete(&mut self, group: &str) -> bool {
        self.groups.remove(group).is_some()
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut g = LayerGroup::new();
        assert!(g.create("").is_err());
        assert!(g.add_member("missing", "fg").is_err());
    }

    #[test]
    fn differential() {
        let mut g = LayerGroup::new();
        g.create("foreground").unwrap();
        g.add_member("foreground", "fg").unwrap();
        g.add_member("foreground", "text").unwrap();
        assert_eq!(g.members("foreground").unwrap().len(), 2);
        g.remove_member("foreground", "text").unwrap();
        assert_eq!(g.members("foreground").unwrap().len(), 1);
        assert!(g.delete("foreground"));
    }
}
