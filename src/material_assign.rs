//! Safe pure MaterialAssign (material-to-bone core).
//!
//! Binds a material name to a bone name. Empty names are
//! rejected. No unwraps.
//!
//! Differential: assign + lookup round-trips.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialAssign {
    map: HashMap<String, String>,
}

impl MaterialAssign {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assign(&mut self, bone: &str, material: &str) -> Result<(), AssignError> {
        if bone.is_empty() || material.is_empty() {
            return Err(AssignError::EmptyName);
        }
        self.map.insert(bone.to_string(), material.to_string());
        Ok(())
    }

    pub fn material_of(&self, bone: &str) -> Result<&str, AssignError> {
        self.map.get(bone).map(|s| s.as_str()).ok_or(AssignError::Missing)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn remove(&mut self, bone: &str) -> bool {
        self.map.remove(bone).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_and_lookup() {
        let mut a = MaterialAssign::new();
        a.assign("plane", "chrome").unwrap();
        assert_eq!(a.material_of("plane").unwrap(), "chrome");
    }

    #[test]
    fn empty_name() {
        let mut a = MaterialAssign::new();
        assert!(a.assign("", "chrome").is_err());
        assert!(a.assign("plane", "").is_err());
    }

    #[test]
    fn differential() {
        let mut a = MaterialAssign::new();
        a.assign("wing", "paint").unwrap();
        a.assign("tail", "metal").unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a.material_of("tail").unwrap(), "metal");
    }
}
