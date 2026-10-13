//! Safe pure GainFilter (audio gain core).
//!
//! Multiplies a sample by a gain. Clamps to -1..=1.
//! No unwraps.
//!
//! Differential: gain 2 doubles a quiet sample.

#[derive(Clone, Debug)]
pub struct GainFilter {
    pub gain: f32,
}

impl Default for GainFilter {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl GainFilter {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    /// Apply gain to one sample. Clamped to -1..=1.
    pub fn apply(&self, sample: f32) -> f32 {
        (sample * self.gain).clamp(-1.0, 1.0)
    }

    /// Apply to a buffer. Returns the new samples.
    pub fn apply_buf(&self, samples: &[f32]) -> Vec<f32> {
        samples.iter().map(|s| self.apply(*s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let g = GainFilter::new(1.0);
        assert_eq!(g.apply(0.5), 0.5);
    }

    #[test]
    fn differential() {
        let g = GainFilter::new(2.0);
        assert_eq!(g.apply(0.25), 0.5);
        assert_eq!(g.apply(0.9), 1.0);
    }
}
