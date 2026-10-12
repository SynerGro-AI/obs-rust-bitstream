//! Safe pure SignalHandler (OBS callback/signal.h core).
//!
//! The C version broadcasts named signals to connected callbacks.
//! The safe path is a HashMap of signal names to listener lists.
//! Unknown signals and empty names are no-ops. No unwraps.
//!
//! Differential: connect + signal delivers the payload.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SignalHandler {
    signals: HashMap<String, Vec<String>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
        }
    }

    /// Declare a signal name. Empty names are ignored.
    pub fn add(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        self.signals.entry(name.to_string()).or_default();
        true
    }

    /// Connect a listener id to a signal. Returns false if the signal was not declared.
    pub fn connect(&mut self, name: &str, listener: &str) -> bool {
        if let Some(list) = self.signals.get_mut(name) {
            if !list.iter().any(|l| l == listener) {
                list.push(listener.to_string());
            }
            true
        } else {
            false
        }
    }

    /// Emit a signal. Returns the list of listener ids that received it.
    pub fn signal(&self, name: &str) -> Vec<String> {
        self.signals
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn listener_count(&self, name: &str) -> usize {
        self.signals.get(name).map(|l| l.len()).unwrap_or(0)
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_connect_signal() {
        let mut h = SignalHandler::new();
        assert!(h.add("source_activate"));
        assert!(h.connect("source_activate", "cb1"));
        assert_eq!(h.signal("source_activate"), vec!["cb1".to_string()]);
    }

    #[test]
    fn unknown_safe() {
        let h = SignalHandler::new();
        assert!(h.signal("missing").is_empty());
        let mut h = SignalHandler::new();
        assert!(!h.add(""));
        assert!(!h.connect("nope", "cb"));
    }

    #[test]
    fn duplicate_connect() {
        let mut h = SignalHandler::new();
        h.add("s");
        h.connect("s", "a");
        h.connect("s", "a");
        assert_eq!(h.listener_count("s"), 1);
    }

    #[test]
    fn differential() {
        let mut h = SignalHandler::new();
        h.add("tick");
        h.connect("tick", "listener");
        assert_eq!(h.signal("tick").len(), 1);
    }
}
