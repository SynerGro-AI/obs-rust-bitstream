//! Safe pure EQ (parametric equalizer core).
//!
//! A band owns a frequency, a gain in dB, and a Q. Empty names
//! are rejected. No unwraps.
//!
//! Differential: a +6 dB band boosts a 1.0 signal above 1.0.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EqError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct EqBand {
    name: String,
    freq_hz: f32,
    gain_db: f32,
    q: f32,
}

impl EqBand {
    pub fn new(name: &str) -> Result<Self, EqError> {
        if name.is_empty() {
            return Err(EqError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            freq_hz: 1000.0,
            gain_db: 0.0,
            q: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_freq(&mut self, hz: f32) {
        self.freq_hz = hz.max(20.0);
    }

    pub fn freq_hz(&self) -> f32 {
        self.freq_hz
    }

    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db = db.clamp(-24.0, 24.0);
    }

    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q.max(0.1);
    }

    pub fn q(&self) -> f32 {
        self.q
    }

    /// Linear gain from the dB setting.
    pub fn linear_gain(&self) -> f32 {
        10.0_f32.powf(self.gain_db / 20.0)
    }

    /// Apply the band to a sample (simple gain; a real filter
    /// would use the frequency and Q).
    pub fn apply(&self, sample: f32) -> f32 {
        sample * self.linear_gain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let b = EqBand::new("low").unwrap();
        assert_eq!(b.freq_hz(), 1000.0);
        assert_eq!(b.gain_db(), 0.0);
    }

    #[test]
    fn empty_name() {
        assert!(EqBand::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut b = EqBand::new("presence").unwrap();
        b.set_gain_db(6.0);
        let out = b.apply(1.0);
        assert!(out > 1.5);
        assert!(out < 2.1);
    }
}
