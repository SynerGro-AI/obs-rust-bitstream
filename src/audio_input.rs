//! Safe pure AudioInput (microphone holder core).
//!
//! Holds a device name and a mute flag. Empty names
//! rejected. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioInputError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct AudioInput {
    device: String,
    muted: bool,
}

impl AudioInput {
    pub fn new(device: &str, muted: bool) -> Result<Self, AudioInputError> {
        if device.is_empty() {
            return Err(AudioInputError::EmptyName);
        }
        Ok(Self {
            device: device.to_string(),
            muted,
        })
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn set_device(&mut self, device: &str) -> Result<(), AudioInputError> {
        if device.is_empty() {
            return Err(AudioInputError::EmptyName);
        }
        self.device = device.to_string();
        Ok(())
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(AudioInput::new("", false).is_err());
    }

    #[test]
    fn differential() {
        let mut a = AudioInput::new("Built-in", false).unwrap();
        assert_eq!(a.device(), "Built-in");
        assert!(!a.muted());
        a.set_device("USB Mic").unwrap();
        a.set_muted(true);
        assert_eq!(a.device(), "USB Mic");
        assert!(a.muted());
    }
}
