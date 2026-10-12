//! Safe pure Profile (OBS profile core).
//!
//! A profile owns a name and a settings bag. Empty names are
//! rejected. No unwraps.
//!
//! Differential: set a setting + get round-trips.

use crate::obs_data::ObsData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Profile {
    name: String,
    settings: ObsData,
}

impl Profile {
    pub fn new(name: &str) -> Result<Self, ProfileError> {
        if name.is_empty() {
            return Err(ProfileError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            settings: ObsData::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn settings(&self) -> &ObsData {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ObsData {
        &mut self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let p = Profile::new("Default").unwrap();
        assert_eq!(p.name(), "Default");
        assert_eq!(p.settings().len(), 0);
    }

    #[test]
    fn empty_name() {
        assert!(Profile::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut p = Profile::new("Gaming").unwrap();
        p.settings_mut().set_int("base_width", 1920).unwrap();
        assert_eq!(p.settings().get_int("base_width").unwrap(), 1920);
    }
}
