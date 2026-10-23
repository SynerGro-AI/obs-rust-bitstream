//! Safe pure ProcHandler (OBS callback/proc.h core).
//!
//! The C version registers named procedures and calls them.
//! The safe path stores parsed declarations (name + params) and
//! a registered id. add_decl accepts the real OBS declaration strings.
//! Unknown names, empty names, and bad decls are rejected. No unwraps.
//!
//! Differential: add_decl + call succeeds for a registered name.

use std::collections::HashMap;

use crate::decl::{parse_decl, DeclError, DeclInfo};

#[derive(Clone, Debug)]
struct ProcSlot {
    decl: DeclInfo,
    id: String,
}

#[derive(Clone, Debug, Default)]
pub struct ProcHandler {
    procs: HashMap<String, ProcSlot>,
}

impl ProcHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a procedure by name and id. Empty names or ids are rejected.
    pub fn add(&mut self, name: &str, id: &str) -> bool {
        if name.is_empty() || id.is_empty() {
            return false;
        }
        self.procs.insert(
            name.to_string(),
            ProcSlot {
                decl: DeclInfo {
                    name: name.to_string(),
                    params: Vec::new(),
                },
                id: id.to_string(),
            },
        );
        true
    }

    /// Register a procedure from an OBS declaration string.
    /// Example: "get_name(string name)"
    /// The id is the registered callback identity.
    pub fn add_decl(&mut self, decl_string: &str, id: &str) -> Result<&str, DeclError> {
        if id.is_empty() {
            return Err(DeclError::Empty);
        }
        let info = parse_decl(decl_string)?;
        let name = info.name.clone();
        self.procs.insert(
            name.clone(),
            ProcSlot {
                decl: info,
                id: id.to_string(),
            },
        );
        Ok(self
            .procs
            .get(&name)
            .map(|s| s.decl.name.as_str())
            .unwrap_or(""))
    }

    /// Call a registered procedure. Returns the id if found.
    pub fn call(&self, name: &str) -> Option<&str> {
        self.procs.get(name).map(|s| s.id.as_str())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.procs.contains_key(name)
    }

    pub fn decl(&self, name: &str) -> Option<&DeclInfo> {
        self.procs.get(name).map(|s| &s.decl)
    }

    /// Remove a registered procedure. Returns true if it existed.
    pub fn remove(&mut self, name: &str) -> bool {
        self.procs.remove(name).is_some()
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
    fn add_decl_parses() {
        let mut h = ProcHandler::new();
        assert!(h.add_decl("get_name(string name)", "proc1").is_ok());
        assert!(h.contains("get_name"));
        let d = h.decl("get_name").unwrap();
        assert_eq!(d.params.len(), 1);
        assert_eq!(d.params[0].name, "name");
        assert_eq!(h.call("get_name"), Some("proc1"));
    }

    #[test]
    fn bad_decl_rejected() {
        let mut h = ProcHandler::new();
        assert!(h.add_decl("bad(foo bar)", "x").is_err());
        assert!(!h.contains("bad"));
    }

    #[test]
    fn empty_id_rejected() {
        let mut h = ProcHandler::new();
        assert!(h.add_decl("get_name(string name)", "").is_err());
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
        h.add_decl("void tick()", "listener").unwrap();
        assert!(h.contains("tick"));
    }

    #[test]
    fn remove_proc() {
        let mut h = ProcHandler::new();
        h.add("p", "id");
        assert!(h.remove("p"));
        assert!(!h.contains("p"));
        assert!(!h.remove("p"));
    }
}
