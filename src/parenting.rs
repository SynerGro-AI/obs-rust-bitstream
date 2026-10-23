//! Safe pure Parenting (layer parent link core).
//!
//! Links a child layer to a parent. Empty names rejected.
//! No unwraps.
//!
//! Differential: set parent + query round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParentError {
    EmptyName,
    Missing,
    Cycle,
}

#[derive(Clone, Debug, Default)]
pub struct Parenting {
    links: BTreeMap<String, String>,
}

impl Parenting {
    pub fn new() -> Self {
        Self::default()
    }

    /// Make `child` parented to `parent`.
    pub fn set(&mut self, child: &str, parent: &str) -> Result<(), ParentError> {
        if child.is_empty() || parent.is_empty() {
            return Err(ParentError::EmptyName);
        }
        if child == parent {
            return Err(ParentError::Cycle);
        }
        // Reject a direct cycle.
        if self.links.get(parent).map(|p| p == child).unwrap_or(false) {
            return Err(ParentError::Cycle);
        }
        self.links.insert(child.to_string(), parent.to_string());
        Ok(())
    }

    pub fn parent_of(&self, child: &str) -> Result<&str, ParentError> {
        self.links
            .get(child)
            .map(|s| s.as_str())
            .ok_or(ParentError::Missing)
    }

    pub fn clear(&mut self, child: &str) -> bool {
        self.links.remove(child).is_some()
    }

    pub fn len(&self) -> usize {
        self.links.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_cycle() {
        let mut p = Parenting::new();
        assert!(p.set("", "root").is_err());
        assert!(p.set("a", "a").is_err());
        p.set("b", "a").unwrap();
        assert!(p.set("a", "b").is_err());
    }

    #[test]
    fn differential() {
        let mut p = Parenting::new();
        p.set("arm", "body").unwrap();
        assert_eq!(p.parent_of("arm").unwrap(), "body");
        assert!(p.clear("arm"));
        assert!(p.parent_of("arm").is_err());
    }
}
