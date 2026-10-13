//! Safe pure NoiseFilter (deterministic noise core).
//!
//! Adds deterministic noise to a pixel. No unwraps.
//!
//! Differential: amount 0 leaves the pixel alone,
//! amount 255 changes it.

#[derive(Clone, Debug, Default)]
pub struct NoiseFilter {
    pub amount: u8,
    pub seed: u64,
}

impl NoiseFilter {
    pub fn new(amount: u8, seed: u64) -> Self {
        Self { amount, seed }
    }

    fn hash(seed: u64, x: usize, y: usize) -> u8 {
        let mut h = seed.wrapping_add(x as u64).wrapping_mul(0x9E3779B97F4A7C15);
        h ^= y as u64;
        h = h.wrapping_mul(0xBF58476D1CE4E5B9);
        (h >> 56) as u8
    }

    /// Apply noise at pixel (x, y). Returns clamped 0..=255.
    pub fn apply(&self, value: u8, x: usize, y: usize) -> u8 {
        if self.amount == 0 {
            return value;
        }
        let n = Self::hash(self.seed, x, y) as i16 - 128;
        let delta = (n * self.amount as i16) / 255;
        (value as i16 + delta).clamp(0, 255) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let n = NoiseFilter::new(0, 1);
        assert_eq!(n.apply(128, 0, 0), 128);
    }

    #[test]
    fn differential() {
        let n = NoiseFilter::new(255, 42);
        let out = n.apply(128, 0, 0);
        assert_ne!(out, 128);
    }
}
