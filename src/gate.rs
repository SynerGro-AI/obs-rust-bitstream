//! Safe pure Gate (noise gate core).
//!
//! Silences samples below a threshold. No unwraps.
//!
//! Differential: a sample of 0.1 with threshold 0.2 becomes 0.

#[derive(Clone, Debug)]
pub struct Gate {
    threshold: f32,
}

impl Gate {
    pub fn new() -> Self {
        Self { threshold: 0.1 }
    }

    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    pub fn apply(&self, sample: f32) -> f32 {
        if sample.abs() < self.threshold {
            0.0
        } else {
            sample
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let g = Gate::new();
        assert_eq!(g.threshold(), 0.1);
    }

    #[test]
    fn differential() {
        let mut g = Gate::new();
        g.set_threshold(0.2);
        assert_eq!(g.apply(0.1), 0.0);
        assert_eq!(g.apply(0.5), 0.5);
    }
}
