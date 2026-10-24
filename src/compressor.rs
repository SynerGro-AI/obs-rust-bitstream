//! Safe pure Compressor (dynamics core).
//!
//! A compressor owns a threshold, a ratio, and a makeup gain.
//! Samples above the threshold are reduced. No unwraps.
//!
//! Differential: a sample of 1.0 with threshold 0.5 and ratio 2
//! is reduced.

#[derive(Clone, Debug)]
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    makeup_db: f32,
}

impl Compressor {
    pub fn new() -> Self {
        Self {
            threshold: 0.5,
            ratio: 2.0,
            makeup_db: 0.0,
        }
    }

    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    pub fn set_ratio(&mut self, r: f32) {
        self.ratio = r.max(1.0);
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_makeup_db(&mut self, db: f32) {
        self.makeup_db = db.clamp(-12.0, 12.0);
    }

    pub fn makeup_db(&self) -> f32 {
        self.makeup_db
    }

    /// Apply compression. Above threshold, excess is divided by ratio.
    pub fn apply(&self, sample: f32) -> f32 {
        let abs = sample.abs();
        let sign = if sample < 0.0 { -1.0 } else { 1.0 };
        let compressed = if abs > self.threshold {
            let excess = abs - self.threshold;
            self.threshold + excess / self.ratio
        } else {
            abs
        };
        let makeup = 10.0_f32.powf(self.makeup_db / 20.0);
        sign * compressed * makeup
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = Compressor::new();
        assert_eq!(c.threshold(), 0.5);
        assert_eq!(c.ratio(), 2.0);
    }

    #[test]
    fn below_threshold_unchanged() {
        let c = Compressor::new();
        assert!((c.apply(0.3) - 0.3).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let c = Compressor::new();
        let out = c.apply(1.0);
        assert!(out < 1.0);
        assert!(out > 0.5);
    }
}
