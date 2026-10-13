//! Safe pure Meter (peak and RMS core).
//!
//! Tracks peak and a running RMS over a window. No unwraps.
//!
//! Differential: a constant 1.0 signal reports peak 1.0 and RMS 1.0.

#[derive(Clone, Debug)]
pub struct Meter {
    peak: f32,
    sum_sq: f32,
    count: usize,
    window: usize,
}

impl Meter {
    pub fn new(window: usize) -> Self {
        Self {
            peak: 0.0,
            sum_sq: 0.0,
            count: 0,
            window: window.max(1),
        }
    }

    pub fn peak(&self) -> f32 {
        self.peak
    }

    /// Root-mean-square of the samples seen so far (up to window).
    pub fn rms(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum_sq / self.count as f32).sqrt()
        }
    }

    /// Feed one sample. Peak holds the max absolute value.
    pub fn push(&mut self, sample: f32) {
        let a = sample.abs();
        if a > self.peak {
            self.peak = a;
        }
        if self.count < self.window {
            self.sum_sq += a * a;
            self.count += 1;
        }
    }

    pub fn reset(&mut self) {
        self.peak = 0.0;
        self.sum_sq = 0.0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = Meter::new(8);
        assert_eq!(m.peak(), 0.0);
        assert_eq!(m.rms(), 0.0);
    }

    #[test]
    fn differential() {
        let mut m = Meter::new(4);
        m.push(1.0);
        m.push(-1.0);
        m.push(0.5);
        assert_eq!(m.peak(), 1.0);
        assert!(m.rms() > 0.7);
        assert!(m.rms() < 1.0);
    }
}
