//! Safe pure Gate (audio gate core).
//!
//! Silences samples below a threshold. No unwraps.
//!
//! Differential: a quiet sample is silenced, a loud one
//! passes through.

#[derive(Clone, Debug)]
pub struct Gate {
    pub threshold: f32,
}

impl Default for Gate {
    fn default() -> Self {
        Self { threshold: 0.1 }
    }
}

impl Gate {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Silence samples below the threshold.
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
    fn loud() {
        let g = Gate::new(0.1);
        assert_eq!(g.apply(0.5), 0.5);
    }

    #[test]
    fn differential() {
        let g = Gate::new(0.1);
        assert_eq!(g.apply(0.05), 0.0);
        assert_eq!(g.apply(-0.05), 0.0);
    }
}
