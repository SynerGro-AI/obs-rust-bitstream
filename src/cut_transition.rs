//! Safe pure CutTransition (hard cut core).
//!
//! Switches from the first to the second at progress 0.5.
//! No unwraps.
//!
//! Differential: below 0.5 is the first, at or above is
//! the second.

pub struct CutTransition;

impl CutTransition {
    /// Pick a pixel. progress is clamped 0..=1.
    pub fn pick(a: u8, b: u8, progress: f32) -> u8 {
        if progress.clamp(0.0, 1.0) < 0.5 {
            a
        } else {
            b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends() {
        assert_eq!(CutTransition::pick(1, 2, 0.0), 1);
        assert_eq!(CutTransition::pick(1, 2, 1.0), 2);
    }

    #[test]
    fn differential() {
        assert_eq!(CutTransition::pick(1, 2, 0.49), 1);
        assert_eq!(CutTransition::pick(1, 2, 0.5), 2);
    }
}
