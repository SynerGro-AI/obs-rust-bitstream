//! Safe pure SignalHandler (OBS callback/signal.h core).
//!
//! The C version broadcasts named signals to connected callbacks.
//! The safe path stores parsed declarations (name + params) and
//! listener lists. add_decl accepts the real OBS declaration strings.
//! Unknown signals, empty names, and bad decls are no-ops / errors.
//! disconnect removes a listener. No unwraps.
//!
//! Differential: add_decl + connect + signal delivers the payload.

use std::collections::HashMap;

use crate::decl::{parse_decl, DeclError, DeclInfo};

#[derive(Clone, Debug)]
struct SignalSlot {
    decl: DeclInfo,
    listeners: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SignalHandler {
    signals: HashMap<String, SignalSlot>,
    globals: Vec<String>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a signal by name only (no params). Empty names are rejected.
    pub fn add(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        self.signals
            .entry(name.to_string())
            .or_insert_with(|| SignalSlot {
                decl: DeclInfo {
                    name: name.to_string(),
                    params: Vec::new(),
                },
                listeners: Vec::new(),
            });
        true
    }

    /// Declare a signal from an OBS declaration string.
    /// Example: "void activate(ptr source)"
    pub fn add_decl(&mut self, decl_string: &str) -> Result<&str, DeclError> {
        let info = parse_decl(decl_string)?;
        let name = info.name.clone();
        self.signals
            .entry(name.clone())
            .or_insert_with(|| SignalSlot {
                decl: info,
                listeners: Vec::new(),
            });
        Ok(self
            .signals
            .get(&name)
            .map(|s| s.decl.name.as_str())
            .unwrap_or(""))
    }

    /// Connect a listener id to a signal. Returns false if the signal was not declared.
    pub fn connect(&mut self, name: &str, listener: &str) -> bool {
        if let Some(slot) = self.signals.get_mut(name) {
            if !slot.listeners.iter().any(|l| l == listener) {
                slot.listeners.push(listener.to_string());
            }
            true
        } else {
            false
        }
    }

    /// Disconnect a listener. Returns true if it was present.
    pub fn disconnect(&mut self, name: &str, listener: &str) -> bool {
        if let Some(slot) = self.signals.get_mut(name) {
            let before = slot.listeners.len();
            slot.listeners.retain(|l| l != listener);
            slot.listeners.len() != before
        } else {
            false
        }
    }

    /// Connect a global listener that receives every signal.
    pub fn connect_global(&mut self, listener: &str) -> bool {
        if listener.is_empty() {
            return false;
        }
        if !self.globals.iter().any(|l| l == listener) {
            self.globals.push(listener.to_string());
        }
        true
    }

    /// Disconnect a global listener. Returns true if it was present.
    pub fn disconnect_global(&mut self, listener: &str) -> bool {
        let before = self.globals.len();
        self.globals.retain(|l| l != listener);
        self.globals.len() != before
    }

    /// Emit a signal. Returns the per-signal listeners.
    /// Global listeners are also notified (see `signal_with_globals`).
    pub fn signal(&self, name: &str) -> Vec<String> {
        self.signals
            .get(name)
            .map(|s| s.listeners.clone())
            .unwrap_or_default()
    }

    /// Emit a signal and return both per-signal listeners and global listeners.
    /// Globals receive every signal name, matching OBS signal_handler_signal.
    pub fn signal_with_globals(&self, name: &str) -> (Vec<String>, Vec<String>) {
        let locals = self.signal(name);
        let globals = if self.has(name) {
            self.globals.clone()
        } else {
            Vec::new()
        };
        (locals, globals)
    }

    pub fn global_count(&self) -> usize {
        self.globals.len()
    }

    pub fn listener_count(&self, name: &str) -> usize {
        self.signals
            .get(name)
            .map(|s| s.listeners.len())
            .unwrap_or(0)
    }

    pub fn has(&self, name: &str) -> bool {
        self.signals.contains_key(name)
    }

    pub fn decl(&self, name: &str) -> Option<&DeclInfo> {
        self.signals.get(name).map(|s| &s.decl)
    }

    /// Remove a declared signal and its listeners. Returns true if it existed.
    pub fn remove(&mut self, name: &str) -> bool {
        self.signals.remove(name).is_some()
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
    fn add_decl_parses() {
        let mut h = SignalHandler::new();
        assert!(h.add_decl("void activate(ptr source)").is_ok());
        assert!(h.has("activate"));
        let d = h.decl("activate").unwrap();
        assert_eq!(d.params.len(), 1);
        assert_eq!(d.params[0].name, "source");
    }

    #[test]
    fn bad_decl_rejected() {
        let mut h = SignalHandler::new();
        assert!(h.add_decl("void bad(foo bar)").is_err());
        assert!(!h.has("bad"));
    }

    #[test]
    fn disconnect() {
        let mut h = SignalHandler::new();
        h.add("s");
        h.connect("s", "a");
        h.connect("s", "b");
        assert!(h.disconnect("s", "a"));
        assert_eq!(h.listener_count("s"), 1);
        assert!(!h.disconnect("s", "missing"));
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
    fn differential() {
        let mut h = SignalHandler::new();
        h.add_decl("void tick()").unwrap();
        h.connect("tick", "listener");
        assert_eq!(h.signal("tick").len(), 1);
    }

    #[test]
    fn global_receives_every_signal() {
        let mut h = SignalHandler::new();
        h.add("activate");
        h.add("deactivate");
        h.connect("activate", "local");
        assert!(h.connect_global("global"));
        let (locals, globals) = h.signal_with_globals("activate");
        assert_eq!(locals, vec!["local".to_string()]);
        assert_eq!(globals, vec!["global".to_string()]);
        let (_, globals2) = h.signal_with_globals("deactivate");
        assert_eq!(globals2, vec!["global".to_string()]);
    }

    #[test]
    fn global_unknown_safe() {
        let mut h = SignalHandler::new();
        h.connect_global("g");
        let (locals, globals) = h.signal_with_globals("missing");
        assert!(locals.is_empty());
        assert!(globals.is_empty());
    }

    #[test]
    fn disconnect_global() {
        let mut h = SignalHandler::new();
        h.connect_global("g");
        assert_eq!(h.global_count(), 1);
        assert!(h.disconnect_global("g"));
        assert_eq!(h.global_count(), 0);
        assert!(!h.disconnect_global("missing"));
    }

    #[test]
    fn remove_signal() {
        let mut h = SignalHandler::new();
        h.add("s");
        h.connect("s", "a");
        assert!(h.remove("s"));
        assert!(!h.has("s"));
        assert!(!h.remove("s"));
    }
}
