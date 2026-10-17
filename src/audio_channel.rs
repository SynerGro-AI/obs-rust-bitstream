//! Safe pure AudioChannel (audio graph core).
//!
//! A channel owns a name, a gain, and a mute flag. Empty names
//! are rejected. No unwraps.
//!
//! Differential: set gain + get round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct AudioChannel {
    name: String,
    gain: f32,
    mute: bool,
    samples: Vec<f32>,
    sample_rate: u32,
}

impl AudioChannel {
    pub fn new(name: &str) -> Result<Self, ChannelError> {
        if name.is_empty() {
            return Err(ChannelError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            gain: 1.0,
            mute: false,
            samples: Vec::new(),
            sample_rate: 44100,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_gain(&mut self, g: f32) {
        self.gain = g.max(0.0);
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    pub fn set_mute(&mut self, m: bool) {
        self.mute = m;
    }

    pub fn mute(&self) -> bool {
        self.mute
    }

    /// Effective gain: 0 if muted, otherwise the set gain.
    pub fn effective_gain(&self) -> f32 {
        if self.mute {
            0.0
        } else {
            self.gain
        }
    }

    /// Set the sample data.
    pub fn set_samples(&mut self, samples: Vec<f32>, rate: u32) {
        self.samples = samples;
        if rate > 0 {
            self.sample_rate = rate;
        }
    }

    /// Sample at a time in seconds. Missing is 0.
    pub fn sample_at(&self, time: f32) -> f32 {
        if self.samples.is_empty() || time < 0.0 {
            return 0.0;
        }
        let idx = (time * self.sample_rate as f32) as usize;
        self.samples.get(idx).copied().unwrap_or(0.0) * self.effective_gain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = AudioChannel::new("mic").unwrap();
        assert_eq!(c.name(), "mic");
        assert_eq!(c.gain(), 1.0);
        assert!(!c.mute());
    }

    #[test]
    fn empty_name() {
        assert!(AudioChannel::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut c = AudioChannel::new("mic").unwrap();
        c.set_gain(0.5);
        assert_eq!(c.effective_gain(), 0.5);
        c.set_mute(true);
        assert_eq!(c.effective_gain(), 0.0);
        c.set_mute(false);
        c.set_gain(1.0);
        c.set_samples(vec![0.0, 0.5, 1.0], 1);
        assert_eq!(c.sample_at(1.0), 0.5);
    }
}
