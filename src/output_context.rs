//! Safe pure OutputContext (OBS output context core).
//!
//! The C version is the shared context every output carries:
//! name, settings, and signal handler. The safe path owns those
//! pieces. Empty names are rejected. No unwraps.
//!
//! Differential: create + settings round-trip.

use crate::obs_data::ObsData;
use crate::signal::SignalHandler;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct OutputContext {
    name: String,
    settings: ObsData,
    signals: SignalHandler,
    active: bool,
}

impl OutputContext {
    /// Create an output context. Empty names are rejected.
    pub fn new(name: &str) -> Result<Self, OutputError> {
        if name.is_empty() {
            return Err(OutputError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            settings: ObsData::new(),
            signals: SignalHandler::new(),
            active: false,
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

    pub fn signals(&self) -> &SignalHandler {
        &self.signals
    }

    pub fn signals_mut(&mut self) -> &mut SignalHandler {
        &mut self.signals
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let o = OutputContext::new("stream").unwrap();
        assert_eq!(o.name(), "stream");
        assert!(!o.is_active());
    }

    #[test]
    fn empty_name() {
        assert!(OutputContext::new("").is_err());
    }

    #[test]
    fn start_stop() {
        let mut o = OutputContext::new("rec").unwrap();
        o.start();
        assert!(o.is_active());
        o.stop();
        assert!(!o.is_active());
    }

    #[test]
    fn differential() {
        let mut o = OutputContext::new("out").unwrap();
        o.settings_mut().set_string("url", "rtmp://x").unwrap();
        o.signals_mut().add_decl("void start(ptr output)").unwrap();
        assert_eq!(o.settings().get_string("url").unwrap(), "rtmp://x");
        assert!(o.signals().has("start"));
    }
}
