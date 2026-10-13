//! Safe pure FadeTransition (crossfade core).
//!
//! Blends two pixels by a progress 0..=1. No unwraps.
//!
//! Differential: progress 0 is the first, 1 is the second,
//! 0.5 is the average.

pub struct FadeTransition;

impl FadeTransition {
    /// Blend two values by progress. progress is clamped 0..=1.
    pub fn blend(a: u8, b: u8, progress: f32) -> u8 {
        let t = progress.clamp(0.0, 1.0);
        let af = a as f32;
        let bf = b as f32;
        (af + (bf - af) * t).round() as u8
    }

    /// Blend two RGB pixels.
    pub fn blend_rgb(a: (u8, u8, u8), b: (u8, u8, u8), progress: f32) -> (u8, u8, u8) {
        (
            Self::blend(a.0, b.0, progress),
            Self::blend(a.1, b.1, progress),
            Self::blend(a.2, b.2, progress),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends() {
        assert_eq!(FadeTransition::blend(0, 255, 0.0), 0);
        assert_eq!(FadeTransition::blend(0, 255, 1.0), 255);
    }

    #[test]
    fn differential() {
        assert_eq!(FadeTransition::blend(0, 100, 0.5), 50);
        let out = FadeTransition::blend_rgb((0, 0, 0), (100, 200, 50), 0.5);
        assert_eq!(out, (50, 100, 25));
    }
}
