//! Safe pure SlideTransition (slide core).
//!
//! Slides the second frame in from the right. Progress
//! 0..=1. No unwraps.
//!
//! Differential: progress 0.5 shows the right half of
//! the first and the left half of the second.

pub struct SlideTransition;

impl SlideTransition {
    /// Pick a pixel. progress is clamped 0..=1.
    /// x is the column, width is the frame width.
    pub fn pick(a: u8, b: u8, x: usize, width: usize, progress: f32) -> u8 {
        if width == 0 {
            return a;
        }
        let t = progress.clamp(0.0, 1.0);
        let offset = ((1.0 - t) * width as f32) as usize;
        if x >= offset {
            b
        } else {
            a
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends() {
        assert_eq!(SlideTransition::pick(1, 2, 0, 4, 0.0), 1);
        assert_eq!(SlideTransition::pick(1, 2, 0, 4, 1.0), 2);
    }

    #[test]
    fn differential() {
        assert_eq!(SlideTransition::pick(1, 2, 0, 4, 0.5), 1);
        assert_eq!(SlideTransition::pick(1, 2, 1, 4, 0.5), 1);
        assert_eq!(SlideTransition::pick(1, 2, 2, 4, 0.5), 2);
        assert_eq!(SlideTransition::pick(1, 2, 3, 4, 0.5), 2);
    }
}
