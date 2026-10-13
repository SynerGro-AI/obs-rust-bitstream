//! Safe pure Limiter (peak ceiling core).
//!
//! Clamps a sample to a ceiling. No unwraps.
//!
//! Differential: a sample of 2.0 with ceiling 1.0 becomes 1.0.

#[derive(Clone, Debug)]
pub struct Limiter {
    ceiling: f32,
}

impl Limiter {
    pub fn new() -> Self {
        Self { ceiling: 1.0 }
    }

    pub fn set_ceiling(&mut self, c: f32) {
        self.ceiling = c.max(0.0);
    }

    pub fn ceiling(&self) -> f32 {
        self.ceiling
    }

    pub fn apply(&self, sample: f32) -> f32 {
        sample.clamp(-self.ceiling, self.ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let l = Limiter::new();
        assert_eq!(l.ceiling(), 1.0);
    }

    #[test]
    fn differential() {
        let l = Limiter::new();
        assert_eq!(l.apply(2.0), 1.0);
        assert_eq!(l.apply(-2.0), -1.0);
        assert_eq!(l.apply(0.5), 0.5);
    }
}
