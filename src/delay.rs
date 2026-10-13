//! Safe pure Delay (echo core).
//!
//! Holds a delay in samples and a feedback amount. No unwraps.
//!
//! Differential: a impulse followed by silence produces the
//! impulse again after the delay.

#[derive(Clone, Debug)]
pub struct Delay {
    delay_samples: usize,
    feedback: f32,
    buffer: Vec<f32>,
    write: usize,
}

impl Delay {
    pub fn new(delay_samples: usize) -> Self {
        let n = delay_samples.max(1);
        Self {
            delay_samples: n,
            feedback: 0.4,
            buffer: vec![0.0; n],
            write: 0,
        }
    }

    pub fn set_feedback(&mut self, f: f32) {
        self.feedback = f.clamp(0.0, 0.95);
    }

    pub fn feedback(&self) -> f32 {
        self.feedback
    }

    /// Process one sample. Returns the delayed + feedback mix.
    pub fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.write];
        self.buffer[self.write] = input + delayed * self.feedback;
        self.write = (self.write + 1) % self.delay_samples;
        delayed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let d = Delay::new(4);
        assert_eq!(d.feedback(), 0.4);
    }

    #[test]
    fn differential() {
        let mut d = Delay::new(2);
        d.set_feedback(0.0);
        assert_eq!(d.process(1.0), 0.0);
        assert_eq!(d.process(0.0), 0.0);
        assert_eq!(d.process(0.0), 1.0);
    }
}
