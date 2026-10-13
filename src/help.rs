//! Safe pure Help (key list core).
//!
//! Lists the bound keys as lines. No unwraps.
//!
//! Differential: three binds produce three lines.

use crate::keybind::Keybind;

pub struct Help;

impl Help {
    /// Lines of "key -> action".
    pub fn list(binds: &Keybind) -> Vec<String> {
        let mut lines = Vec::new();
        for key in ["r", "s", "v", "R", "S", "V"] {
            if let Some(action) = binds.action(key) {
                lines.push(format!("{key} -> {action}"));
            }
        }
        lines
    }

    pub fn text(binds: &Keybind) -> String {
        Self::list(binds).join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut b = Keybind::new();
        b.bind("r", "record").unwrap();
        b.bind("s", "stream").unwrap();
        let lines = Help::list(&b);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("record"));
    }
}
