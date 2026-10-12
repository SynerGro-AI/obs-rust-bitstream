//! Safe pure CameraRig (multi-camera core).
//!
//! Holds more than one named camera. Empty and duplicate names
//! are rejected. No unwraps.
//!
//! Differential: add two cameras + active switch round-trips.

use std::collections::HashMap;

use crate::camera::Camera;
use crate::vec3::Vec3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraRigError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct CameraRig {
    cameras: HashMap<String, Camera>,
    active: Option<String>,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            active: None,
        }
    }
}

impl CameraRig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), CameraRigError> {
        if name.is_empty() {
            return Err(CameraRigError::EmptyName);
        }
        if self.cameras.contains_key(name) {
            return Err(CameraRigError::Duplicate);
        }
        let cam = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        self.cameras.insert(name.to_string(), cam);
        if self.active.is_none() {
            self.active = Some(name.to_string());
        }
        Ok(())
    }

    pub fn set_active(&mut self, name: &str) -> Result<(), CameraRigError> {
        if !self.cameras.contains_key(name) {
            return Err(CameraRigError::Missing);
        }
        self.active = Some(name.to_string());
        Ok(())
    }

    pub fn active_name(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn active(&self) -> Option<&Camera> {
        self.active.as_ref().and_then(|n| self.cameras.get(n))
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Camera, CameraRigError> {
        self.cameras.get_mut(name).ok_or(CameraRigError::Missing)
    }

    pub fn len(&self) -> usize {
        self.cameras.len()
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let removed = self.cameras.remove(name).is_some();
        if removed && self.active.as_deref() == Some(name) {
            self.active = self.cameras.keys().next().cloned();
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut r = CameraRig::new();
        r.add("main").unwrap();
        r.add("closeup").unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r.active_name(), Some("main"));
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = CameraRig::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(CameraRigError::Duplicate)));
        assert!(matches!(r.add(""), Err(CameraRigError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut r = CameraRig::new();
        r.add("wide").unwrap();
        r.add("hero").unwrap();
        r.set_active("hero").unwrap();
        assert_eq!(r.active_name(), Some("hero"));
        assert!(r.active().is_some());
    }
}
