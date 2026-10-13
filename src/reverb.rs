//! Safe pure Reverb (simple feedback reverb core).
//!
//! A short multi-tap feedback that adds space. No unwraps.
//!
//! Differential: an impulse produces a non-zero tail.

#[derive(Clone, Debug)]
pub struct Reverb {
    taps: Vec<f32>,
    feedback: f32,
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            taps: vec![0.0; 8],
            feedback: 0.5,
        }
    }

    pub fn set_feedback(&mut self, f: f32) {
        self.feedback = f.clamp(0.0, 0.9);
    }

    pub fn feedback(&self) -> f32 {
        self.feedback
    }

    /// Process one sample. Mixes the input with a decaying tail.
    pub fn process(&mut self, input: f32) -> f32 {
        let tail: f32 = self.taps.iter().sum::<f32>() / self.taps.len() as f32;
        let out = input + tail * self.feedback;
        self.taps.rotate_right(1);
        if let Some(slot) = self.taps.first_mut() {
            *slot = out * 0.5;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = Reverb::new();
        assert_eq!(r.feedback(), 0.5);
    }

    #[test]
    fn differential() {
        let mut r = Reverb::new();
        let a = r.process(1.0);
        let b = r.process(0.0);
        assert!(a > 0.9);
        assert!(b > 0.0);
    }
}
