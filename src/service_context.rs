//! Safe pure ServiceContext (OBS service context core).
//!
//! The C version is the shared context every streaming service carries:
//! name, settings, and a service id. The safe path owns those pieces.
//! Empty names are rejected. No unwraps.
//!
//! Differential: create + settings round-trip.

use crate::obs_data::ObsData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct ServiceContext {
    name: String,
    id: String,
    settings: ObsData,
}

impl ServiceContext {
    /// Create a service context. Empty names are rejected.
    pub fn new(name: &str, id: &str) -> Result<Self, ServiceError> {
        if name.is_empty() {
            return Err(ServiceError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            id: id.to_string(),
            settings: ObsData::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
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
        let s = ServiceContext::new("Twitch", "rtmp_common").unwrap();
        assert_eq!(s.name(), "Twitch");
        assert_eq!(s.id(), "rtmp_common");
    }

    #[test]
    fn empty_name() {
        assert!(ServiceContext::new("", "x").is_err());
    }

    #[test]
    fn differential() {
        let mut s = ServiceContext::new("YouTube", "rtmp_common").unwrap();
        s.settings_mut().set_string("key", "abc").unwrap();
        assert_eq!(s.settings().get_string("key").unwrap(), "abc");
    }
}
