//! Safe pure ProcHandler (OBS callback/proc.h core).
//!
//! The C version registers named procedures and calls them.
//! The safe path is a HashMap of names to registered ids.
//! Unknown names return false. No unwraps.
//!
//! Differential: add + call succeeds for a registered name.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ProcHandler {
    procs: HashMap<String, String>,
}

impl ProcHandler {
    pub fn new() -> Self {
        Self {
            procs: HashMap::new(),
        }
    }

    /// Register a procedure name. Empty names are ignored.
    pub fn add(&mut self, name: &str, id: &str) -> bool {
        if name.is_empty() || id.is_empty() {
            return false;
        }
        self.procs.insert(name.to_string(), id.to_string());
        true
    }

    /// Call a registered procedure. Returns the id if found.
    pub fn call(&self, name: &str) -> Option<&str> {
        self.procs.get(name).map(|s| s.as_str())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.procs.contains_key(name)
    }
}

impl Default for ProcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_call() {
        let mut h = ProcHandler::new();
        assert!(h.add("get_name", "proc1"));
        assert_eq!(h.call("get_name"), Some("proc1"));
    }

    #[test]
    fn unknown_safe() {
        let h = ProcHandler::new();
        assert!(h.call("missing").is_none());
        let mut h = ProcHandler::new();
        assert!(!h.add("", "x"));
        assert!(!h.add("n", ""));
    }

    #[test]
    fn overwrite() {
        let mut h = ProcHandler::new();
        h.add("p", "a");
        h.add("p", "b");
        assert_eq!(h.call("p"), Some("b"));
    }

    #[test]
    fn differential() {
        let mut h = ProcHandler::new();
        h.add("tick", "listener");
        assert!(h.contains("tick"));
    }
}
