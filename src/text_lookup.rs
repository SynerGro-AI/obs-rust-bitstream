//! Minimal text lookup (OBS util/text-lookup core idea).
//!
//! Safe HashMap-based store. File loading is left as a future OS-dependent
//! shim; the core map + escape conversion is pure and tested.
//!
//! Quirk: escapes \\n \\t \\r \\" are expanded; comments (#) and blank lines
//! are skipped by the real parser (not re-implemented here).

use std::collections::HashMap;

/// Safe text lookup.
#[derive(Debug, Default)]
pub struct TextLookup {
    items: HashMap<String, String>,
}

impl TextLookup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.items.insert(key.into(), expand_escapes(&value.into()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.items.get(key).map(|s| s.as_str())
    }
}

fn expand_escapes(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::assert_equal;

    #[test]
    fn basic_lookup() {
        let mut t = TextLookup::new();
        t.insert("hello", "world");
        assert_eq!(t.get("hello"), Some("world"));
        assert_eq!(t.get("missing"), None);
    }

    #[test]
    fn escapes() {
        let mut t = TextLookup::new();
        t.insert("nl", "line1\\nline2");
        assert_eq!(t.get("nl"), Some("line1\nline2"));
    }

    #[test]
    fn differential_insert() {
        let mut safe = TextLookup::new();
        safe.insert("k", "v");
        // trivial path
        let abi = "v";
        assert_equal(safe.get("k").unwrap().as_bytes(), abi.as_bytes(), "insert/get");
    }
}
