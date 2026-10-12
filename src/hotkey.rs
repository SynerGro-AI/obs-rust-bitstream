//! Safe pure Hotkey (OBS hotkey core).
//!
//! A hotkey owns a name, an optional description, and a pressed
//! flag. Empty names are rejected. No unwraps.
//!
//! Differential: register + press round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Hotkey {
    name: String,
    description: String,
    pressed: bool,
}

impl Hotkey {
    pub fn new(name: &str, description: &str) -> Result<Self, HotkeyError> {
        if name.is_empty() {
            return Err(HotkeyError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            description: description.to_string(),
            pressed: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn set_pressed(&mut self, v: bool) {
        self.pressed = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let h = Hotkey::new("start_stream", "Start Streaming").unwrap();
        assert_eq!(h.name(), "start_stream");
        assert!(!h.pressed());
    }

    #[test]
    fn empty_name() {
        assert!(Hotkey::new("", "x").is_err());
    }

    #[test]
    fn differential() {
        let mut h = Hotkey::new("mute", "Mute").unwrap();
        h.set_pressed(true);
        assert!(h.pressed());
        h.set_pressed(false);
        assert!(!h.pressed());
    }
}
