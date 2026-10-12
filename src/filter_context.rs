//! Safe pure FilterContext (OBS filter core).
//!
//! A filter owns a name, a filter id, and settings. Empty names
//! are rejected. No unwraps.
//!
//! Differential: create + settings round-trip.

use crate::obs_data::ObsData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct FilterContext {
    name: String,
    id: String,
    settings: ObsData,
    enabled: bool,
}

impl FilterContext {
    /// Create a filter. Empty names are rejected.
    pub fn new(name: &str, id: &str) -> Result<Self, FilterError> {
        if name.is_empty() {
            return Err(FilterError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            id: id.to_string(),
            settings: ObsData::new(),
            enabled: true,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn type_id(&self) -> &str {
        &self.id
    }

    pub fn settings(&self) -> &ObsData {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ObsData {
        &mut self.settings
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let f = FilterContext::new("blur", "gpu_blur").unwrap();
        assert_eq!(f.name(), "blur");
        assert_eq!(f.id(), "gpu_blur");
        assert!(f.enabled());
    }

    #[test]
    fn empty_name() {
        assert!(FilterContext::new("", "id").is_err());
    }

    #[test]
    fn differential() {
        let mut f = FilterContext::new("gain", "gain_filter").unwrap();
        f.settings_mut().set_double("db", -6.0).unwrap();
        f.set_enabled(false);
        assert_eq!(f.settings().get_double("db").unwrap(), -6.0);
        assert!(!f.enabled());
    }
}
