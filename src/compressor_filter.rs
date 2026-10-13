//! Safe pure Compressor (audio compressor core).
//!
//! Reduces samples above a threshold by a ratio. No unwraps.
//!
//! Differential: a loud sample above threshold is reduced,
//! a quiet one is left alone.

#[derive(Clone, Debug)]
pub struct Compressor {
    pub threshold: f32,
    pub ratio: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            ratio: 4.0,
        }
    }
}

impl Compressor {
    pub fn new(threshold: f32, ratio: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            ratio: ratio.max(1.0),
        }
    }

    /// Apply compression to one sample.
    pub fn apply(&self, sample: f32) -> f32 {
        let sign = sample.signum();
        let abs = sample.abs();
        if abs <= self.threshold {
            sample
        } else {
            let over = abs - self.threshold;
            let reduced = self.threshold + over / self.ratio;
            sign * reduced.clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet() {
        let c = Compressor::new(0.5, 4.0);
        assert_eq!(c.apply(0.25), 0.25);
    }

    #[test]
    fn differential() {
        let c = Compressor::new(0.5, 4.0);
        let out = c.apply(0.9);
        assert!(out < 0.9);
        assert!(out > 0.5);
    }
}
