//! Safe pure VirtualCamRegistry (OBS virtual camera lookup core).
//!
//! Name-to-VirtualCam map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + start through the registry round-trips.

use std::collections::HashMap;

use crate::virtual_cam::{VirtualCam, VirtualCamError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualCamRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<VirtualCamError> for VirtualCamRegistryError {
    fn from(_: VirtualCamError) -> Self {
        VirtualCamRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct VirtualCamRegistry {
    cams: HashMap<String, VirtualCam>,
}

impl VirtualCamRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), VirtualCamRegistryError> {
        if name.is_empty() {
            return Err(VirtualCamRegistryError::EmptyName);
        }
        if self.cams.contains_key(name) {
            return Err(VirtualCamRegistryError::Duplicate);
        }
        let c = VirtualCam::new(name)?;
        self.cams.insert(name.to_string(), c);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&VirtualCam, VirtualCamRegistryError> {
        self.cams.get(name).ok_or(VirtualCamRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut VirtualCam, VirtualCamRegistryError> {
        self.cams
            .get_mut(name)
            .ok_or(VirtualCamRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.cams.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.cams.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.cams.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = VirtualCamRegistry::new();
        r.add("main").unwrap();
        assert_eq!(r.get("main").unwrap().name(), "main");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = VirtualCamRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(
            r.add("a"),
            Err(VirtualCamRegistryError::Duplicate)
        ));
        assert!(matches!(r.add(""), Err(VirtualCamRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = VirtualCamRegistry::new();
        assert!(matches!(
            r.get("nope"),
            Err(VirtualCamRegistryError::Missing)
        ));
        r.add("c").unwrap();
        assert!(r.remove("c"));
        assert!(!r.contains("c"));
    }

    #[test]
    fn differential() {
        let mut r = VirtualCamRegistry::new();
        r.add("cam").unwrap();
        r.get_mut("cam").unwrap().start();
        assert!(r.get("cam").unwrap().active());
    }
}
