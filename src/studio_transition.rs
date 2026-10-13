//! Safe pure StudioTransition (program/preview fade core).
//!
//! Owns a progress from 0 (preview) to 1 (program). step
//! advances it. No unwraps.
//!
//! Differential: step to 1 completes.

#[derive(Clone, Debug)]
pub struct StudioTransition {
    progress: f32,
}

impl Default for StudioTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioTransition {
    pub fn new() -> Self {
        Self { progress: 0.0 }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn start(&mut self) {
        self.progress = 0.0;
    }

    /// Advance by `dt`. Returns true when complete.
    pub fn step(&mut self, dt: f32) -> bool {
        self.progress = (self.progress + dt).clamp(0.0, 1.0);
        self.progress >= 1.0
    }

    pub fn done(&self) -> bool {
        self.progress >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let t = StudioTransition::new();
        assert_eq!(t.progress(), 0.0);
    }

    #[test]
    fn differential() {
        let mut t = StudioTransition::new();
        assert!(!t.step(0.4));
        assert!(t.step(0.7));
        assert!(t.done());
    }
}
