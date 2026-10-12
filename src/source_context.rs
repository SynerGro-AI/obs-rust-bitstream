//! Safe pure SourceContext (OBS source context core).
//!
//! The C version is the shared context every source carries:
//! name, settings, properties, signal handler, and proc handler.
//! The safe path owns those pieces. Empty names are rejected.
//! No unwraps.
//!
//! Differential: create + get name / settings round-trips.

use crate::obs_data::ObsData;
use crate::obs_properties::Properties;
use crate::proc::ProcHandler;
use crate::signal::SignalHandler;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct SourceContext {
    name: String,
    settings: ObsData,
    properties: Properties,
    signals: SignalHandler,
    procs: ProcHandler,
}

impl SourceContext {
    /// Create a context. Empty names are rejected.
    pub fn new(name: &str) -> Result<Self, SourceError> {
        if name.is_empty() {
            return Err(SourceError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            settings: ObsData::new(),
            properties: Properties::new(),
            signals: SignalHandler::new(),
            procs: ProcHandler::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: &str) -> Result<(), SourceError> {
        if name.is_empty() {
            return Err(SourceError::EmptyName);
        }
        self.name = name.to_string();
        Ok(())
    }

    pub fn settings(&self) -> &ObsData {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ObsData {
        &mut self.settings
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut Properties {
        &mut self.properties
    }

    pub fn signals(&self) -> &SignalHandler {
        &self.signals
    }

    pub fn signals_mut(&mut self) -> &mut SignalHandler {
        &mut self.signals
    }

    pub fn procs(&self) -> &ProcHandler {
        &self.procs
    }

    pub fn procs_mut(&mut self) -> &mut ProcHandler {
        &mut self.procs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_get() {
        let s = SourceContext::new("Camera").unwrap();
        assert_eq!(s.name(), "Camera");
        assert_eq!(s.settings().len(), 0);
    }

    #[test]
    fn empty_name() {
        assert!(SourceContext::new("").is_err());
    }

    #[test]
    fn rename() {
        let mut s = SourceContext::new("old").unwrap();
        s.rename("new").unwrap();
        assert_eq!(s.name(), "new");
        assert!(s.rename("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = SourceContext::new("src").unwrap();
        s.settings_mut().set_int("width", 1920).unwrap();
        s.signals_mut().add_decl("void activate(ptr source)").unwrap();
        assert_eq!(s.settings().get_int("width").unwrap(), 1920);
        assert!(s.signals().has("activate"));
    }
}
