//! Safe pure ToolOptionSet (adjustable options bar).
//!
//! Holds the current values for the active tool.
//! set changes a value. Unknown names are rejected.
//! Switching tools resets to that tool's defaults.
//! No unwraps.
//!
//! Differential: a larger size reads as a larger value.

use crate::active_tool::{ActiveTool, ToolSource};
use crate::tool_options::ToolOptions;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ToolOptionSet {
    values: HashMap<String, f32>,
}

impl ToolOptionSet {
    /// Defaults for the active tool.
    pub fn for_tool(active: &ActiveTool) -> Self {
        let mut values = HashMap::new();
        for opt in ToolOptions::for_tool(active) {
            values.insert(opt.name.to_string(), opt.value);
        }
        Self { values }
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.values.get(name).copied()
    }

    /// Set a known option. Returns false if the name is unknown.
    pub fn set(&mut self, name: &str, value: f32) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    pub fn line(&self) -> String {
        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();
        keys.iter()
            .map(|k| format!("{}={}", k, self.values[*k]))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brush() -> ActiveTool {
        let mut t = ActiveTool::new();
        t.activate("Brush", ToolSource::Photoshop);
        t
    }

    #[test]
    fn set_changes() {
        let mut opts = ToolOptionSet::for_tool(&brush());
        assert_eq!(opts.get("size"), Some(20.0));
        assert!(opts.set("size", 40.0));
        assert_eq!(opts.get("size"), Some(40.0));
    }

    #[test]
    fn unknown_rejected() {
        let mut opts = ToolOptionSet::for_tool(&brush());
        assert!(!opts.set("not_an_option", 1.0));
    }

    #[test]
    fn differential() {
        let mut a = ToolOptionSet::for_tool(&brush());
        let mut b = ToolOptionSet::for_tool(&brush());
        a.set("size", 10.0);
        b.set("size", 50.0);
        assert!(b.get("size").unwrap() > a.get("size").unwrap());
    }
}
