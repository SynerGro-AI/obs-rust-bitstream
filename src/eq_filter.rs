//! Safe pure Eq (three-band equalizer core).
//!
//! Applies low, mid, and high gains. No unwraps.
//!
//! Differential: a low boost lifts a low-band sample.

#[derive(Clone, Debug)]
pub struct Eq {
    pub low: f32,
    pub mid: f32,
    pub high: f32,
}

impl Default for Eq {
    fn default() -> Self {
        Self {
            low: 1.0,
            mid: 1.0,
            high: 1.0,
        }
    }
}

impl Eq {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the band gain. band is 0 low, 1 mid, 2 high.
    pub fn apply(&self, sample: f32, band: u8) -> f32 {
        let g = match band {
            0 => self.low,
            1 => self.mid,
            _ => self.high,
        };
        (sample * g).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let e = Eq::new();
        assert_eq!(e.apply(0.5, 0), 0.5);
    }

    #[test]
    fn differential() {
        let mut e = Eq::new();
        e.low = 2.0;
        assert_eq!(e.apply(0.25, 0), 0.5);
        assert_eq!(e.apply(0.25, 1), 0.25);
    }
}
