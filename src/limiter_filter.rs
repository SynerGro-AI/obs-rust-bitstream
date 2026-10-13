//! Safe pure Limiter (audio limiter core).
//!
//! Clamps a sample to a ceiling. No unwraps.
//!
//! Differential: a sample above the ceiling is clamped,
//! one below is left alone.

#[derive(Clone, Debug)]
pub struct Limiter {
    pub ceiling: f32,
}

impl Default for Limiter {
    fn default() -> Self {
        Self { ceiling: 0.9 }
    }
}

impl Limiter {
    pub fn new(ceiling: f32) -> Self {
        Self {
            ceiling: ceiling.clamp(0.0, 1.0),
        }
    }

    /// Clamp one sample to the ceiling.
    pub fn apply(&self, sample: f32) -> f32 {
        sample.clamp(-self.ceiling, self.ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet() {
        let l = Limiter::new(0.9);
        assert_eq!(l.apply(0.5), 0.5);
    }

    #[test]
    fn differential() {
        let l = Limiter::new(0.9);
        assert_eq!(l.apply(1.5), 0.9);
        assert_eq!(l.apply(-1.5), -0.9);
    }
}
