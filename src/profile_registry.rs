//! Safe pure ProfileRegistry (OBS profile lookup core).
//!
//! Name-to-Profile map. Empty and duplicate names are rejected.
//! No unwraps.
//!
//! Differential: add + set a setting through the registry round-trips.

use std::collections::HashMap;

use crate::profile::{Profile, ProfileError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<ProfileError> for ProfileRegistryError {
    fn from(_: ProfileError) -> Self {
        ProfileRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProfileRegistry {
    profiles: HashMap<String, Profile>,
}

impl ProfileRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), ProfileRegistryError> {
        if name.is_empty() {
            return Err(ProfileRegistryError::EmptyName);
        }
        if self.profiles.contains_key(name) {
            return Err(ProfileRegistryError::Duplicate);
        }
        let p = Profile::new(name)?;
        self.profiles.insert(name.to_string(), p);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Profile, ProfileRegistryError> {
        self.profiles.get(name).ok_or(ProfileRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Profile, ProfileRegistryError> {
        self.profiles
            .get_mut(name)
            .ok_or(ProfileRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.profiles.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = ProfileRegistry::new();
        r.add("Default").unwrap();
        assert_eq!(r.get("Default").unwrap().name(), "Default");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = ProfileRegistry::new();
        r.add("a").unwrap();
        assert!(matches!(r.add("a"), Err(ProfileRegistryError::Duplicate)));
        assert!(matches!(r.add(""), Err(ProfileRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = ProfileRegistry::new();
        assert!(matches!(r.get("nope"), Err(ProfileRegistryError::Missing)));
        r.add("p").unwrap();
        assert!(r.remove("p"));
        assert!(!r.contains("p"));
    }

    #[test]
    fn differential() {
        let mut r = ProfileRegistry::new();
        r.add("Gaming").unwrap();
        r.get_mut("Gaming")
            .unwrap()
            .settings_mut()
            .set_int("base_width", 1920)
            .unwrap();
        assert_eq!(
            r.get("Gaming")
                .unwrap()
                .settings()
                .get_int("base_width")
                .unwrap(),
            1920
        );
    }
}
