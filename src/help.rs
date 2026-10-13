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

    /// Hover description for a key, Photoshop style.
    pub fn hover(key: &str) -> &'static str {
        match key {
            "r" => "Start recording the program output (R to stop)",
            "s" => "Start streaming to the default target (S to stop)",
            "v" => "Start the virtual camera (V to stop)",
            "R" => "Stop recording",
            "S" => "Stop streaming",
            "V" => "Stop the virtual camera",
            "?" => "Show this help",
            _ => "No description",
        }
    }

    /// Hover lines for every bound key.
    pub fn hovers(binds: &Keybind) -> Vec<String> {
        let mut lines = Vec::new();
        for key in ["r", "s", "v", "R", "S", "V"] {
            if binds.action(key).is_some() {
                lines.push(format!("{key}: {}", Self::hover(key)));
            }
        }
        lines
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

    #[test]
    fn hover() {
        assert!(Help::hover("r").contains("recording"));
        let mut b = Keybind::new();
        b.bind("r", "record").unwrap();
        let hovers = Help::hovers(&b);
        assert_eq!(hovers.len(), 1);
        assert!(hovers[0].contains("recording"));
    }
}
