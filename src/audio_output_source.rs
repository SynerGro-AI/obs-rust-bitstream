//! Safe pure AudioOutput (desktop audio holder core).
//!
//! Holds a device name and a mute flag. Empty names
//! rejected. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioOutputError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct AudioOutput {
    device: String,
    muted: bool,
}

impl AudioOutput {
    pub fn new(device: &str, muted: bool) -> Result<Self, AudioOutputError> {
        if device.is_empty() {
            return Err(AudioOutputError::EmptyName);
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

    pub fn set_device(&mut self, device: &str) -> Result<(), AudioOutputError> {
        if device.is_empty() {
            return Err(AudioOutputError::EmptyName);
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
        assert!(AudioOutput::new("", false).is_err());
    }

    #[test]
    fn differential() {
        let mut a = AudioOutput::new("Speakers", false).unwrap();
        assert_eq!(a.device(), "Speakers");
        assert!(!a.muted());
        a.set_device("Headphones").unwrap();
        a.set_muted(true);
        assert_eq!(a.device(), "Headphones");
        assert!(a.muted());
    }
}
