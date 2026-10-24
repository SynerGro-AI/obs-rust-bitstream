//! Safe pure AudioMonitor (OBS audio monitoring core).
//!
//! A monitor owns a name, a volume (0.0–1.0), and a muted flag.
//! Empty names are rejected. Volume is clamped. No unwraps.
//!
//! Differential: set volume + get round-trips.

#[derive(Clone, Debug, PartialEq)]
pub enum MonitorError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct AudioMonitor {
    name: String,
    volume: f32,
    muted: bool,
}

impl AudioMonitor {
    pub fn new(name: &str) -> Result<Self, MonitorError> {
        if name.is_empty() {
            return Err(MonitorError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            volume: 1.0,
            muted: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Set volume, clamped to 0.0–1.0.
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn set_muted(&mut self, v: bool) {
        self.muted = v;
    }

    /// Effective gain: 0 if muted, otherwise volume.
    pub fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = AudioMonitor::new("desktop").unwrap();
        assert_eq!(m.name(), "desktop");
        assert_eq!(m.gain(), 1.0);
    }

    #[test]
    fn empty_name() {
        assert!(AudioMonitor::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut m = AudioMonitor::new("mic").unwrap();
        m.set_volume(0.5);
        m.set_muted(true);
        assert_eq!(m.gain(), 0.0);
        m.set_muted(false);
        assert_eq!(m.gain(), 0.5);
        m.set_volume(2.0);
        assert_eq!(m.volume(), 1.0);
    }
}
