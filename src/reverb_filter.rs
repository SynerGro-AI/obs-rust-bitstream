//! Safe pure Reverb (simple delay reverb core).
//!
//! Mixes a delayed copy of the sample. No unwraps.
//!
//! Differential: a delay of 2 mixes the sample from
//! two ticks ago.

#[derive(Clone, Debug)]
pub struct Reverb {
    delay: usize,
    mix: f32,
    buf: Vec<f32>,
    pos: usize,
}

impl Reverb {
    pub fn new(delay: usize, mix: f32) -> Self {
        let delay = delay.max(1);
        Self {
            delay,
            mix: mix.clamp(0.0, 1.0),
            buf: vec![0.0; delay],
            pos: 0,
        }
    }

    /// Process one sample. Returns the mixed output.
    pub fn apply(&mut self, sample: f32) -> f32 {
        let delayed = self.buf[self.pos];
        let out = sample * (1.0 - self.mix) + delayed * self.mix;
        self.buf[self.pos] = sample;
        self.pos = (self.pos + 1) % self.delay;
        out.clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = Reverb::new(2, 0.5);
        assert_eq!(r.delay, 2);
    }

    #[test]
    fn differential() {
        let mut r = Reverb::new(2, 0.5);
        let a = r.apply(1.0);
        let b = r.apply(0.0);
        let c = r.apply(0.0);
        assert!((a - 0.5).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.01);
        assert!((c - 0.5).abs() < 0.01);
    }
}
