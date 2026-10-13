//! Safe pure GroupSource (source group core).
//!
//! Holds a name and a list of child source names. Empty
//! names rejected. No unwraps.
//!
//! Differential: add + list + remove round-trips.

use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug)]
pub struct GroupSource {
    name: String,
    children: BTreeSet<String>,
}

impl GroupSource {
    pub fn new(name: &str) -> Result<Self, GroupError> {
        if name.is_empty() {
            return Err(GroupError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            children: BTreeSet::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, child: &str) -> Result<(), GroupError> {
        if child.is_empty() {
            return Err(GroupError::EmptyName);
        }
        self.children.insert(child.to_string());
        Ok(())
    }

    pub fn remove(&mut self, child: &str) -> Result<(), GroupError> {
        if self.children.remove(child) {
            Ok(())
        } else {
            Err(GroupError::Missing)
        }
    }

    pub fn children(&self) -> Vec<&str> {
        self.children.iter().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.children.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(GroupSource::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut g = GroupSource::new("overlay").unwrap();
        g.add("logo").unwrap();
        g.add("text").unwrap();
        g.add("logo").unwrap();
        assert_eq!(g.count(), 2);
        assert_eq!(g.children(), vec!["logo", "text"]);
        g.remove("logo").unwrap();
        assert_eq!(g.count(), 1);
    }
}
