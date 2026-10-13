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

    /// Hover description for a key, instructive style.
    pub fn hover(key: &str) -> &'static str {
        match key {
            "r" => "Press r to start recording. The program output is captured frame by frame. Press R to stop. The recording is saved as a clip.",
            "s" => "Press s to start streaming. The program output is sent to the default target. Press S to stop. The stream is live while this is on.",
            "v" => "Press v to start the virtual camera. Other apps see the program as a camera. Press V to stop.",
            "?" => "Press ? to see every bound key and what it does.",
            "R" => "Press R to stop recording. The clip is kept.",
            "S" => "Press S to stop streaming. The live feed ends.",
            "V" => "Press V to stop the virtual camera. Other apps no longer see it.",
            _ => "No description for this key.",
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
