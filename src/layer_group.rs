//! Safe pure LayerGroup (Photoshop-style group core).
//!
//! SUPERSEDED by layer_group_pure, which adds remove_member
//! and delete. Kept so existing callers still compile.
//!
//! A group owns a name and a list of member layer names.
//! Empty names rejected. No unwraps.
//!
//! Differential: add two members + list round-trips.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct LayerGroup {
    name: String,
    members: Vec<String>,
}

impl LayerGroup {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn members(&self) -> &[String] {
        &self.members
    }
}

#[derive(Clone, Debug, Default)]
pub struct GroupSet {
    groups: HashMap<String, LayerGroup>,
}

impl GroupSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), GroupError> {
        if name.is_empty() {
            return Err(GroupError::EmptyName);
        }
        if self.groups.contains_key(name) {
            return Err(GroupError::Duplicate);
        }
        self.groups.insert(
            name.to_string(),
            LayerGroup {
                name: name.to_string(),
                members: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn add_member(&mut self, group: &str, layer: &str) -> Result<(), GroupError> {
        let g = self.groups.get_mut(group).ok_or(GroupError::Missing)?;
        if !g.members.iter().any(|m| m == layer) {
            g.members.push(layer.to_string());
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&LayerGroup, GroupError> {
        self.groups.get(name).ok_or(GroupError::Missing)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_members() {
        let mut s = GroupSet::new();
        s.add("foreground").unwrap();
        s.add_member("foreground", "fg").unwrap();
        s.add_member("foreground", "text").unwrap();
        assert_eq!(s.get("foreground").unwrap().members().len(), 2);
    }

    #[test]
    fn empty_name() {
        let mut s = GroupSet::new();
        assert!(s.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = GroupSet::new();
        s.add("g").unwrap();
        s.add_member("g", "a").unwrap();
        assert_eq!(s.get("g").unwrap().members()[0], "a");
    }
}
