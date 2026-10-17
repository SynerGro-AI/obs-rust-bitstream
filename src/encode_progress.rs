//! Safe pure EncodeProgress (frames done core).
//!
//! Tracks how many frames have been encoded.
//! No unwraps.
//!
//! Differential: tick advances the count.

#[derive(Clone, Debug, Default)]
pub struct EncodeProgress {
    done: usize,
    total: usize,
}

impl EncodeProgress {
    pub fn new(total: usize) -> Self {
        Self { done: 0, total }
    }

    pub fn done(&self) -> usize {
        self.done
    }

    pub fn total(&self) -> usize {
        self.total
    }

    /// Advance by one. Stops at total.
    pub fn tick(&mut self) {
        if self.done < self.total {
            self.done += 1;
        }
    }

    /// Fraction complete, 0.0..=1.0.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.done as f32 / self.total as f32
    }

    pub fn is_done(&self) -> bool {
        self.done >= self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut p = EncodeProgress::new(4);
        assert_eq!(p.fraction(), 0.0);
        p.tick();
        p.tick();
        assert_eq!(p.done(), 2);
        assert!((p.fraction() - 0.5).abs() < 0.01);
        p.tick();
        p.tick();
        assert!(p.is_done());
        p.tick();
        assert_eq!(p.done(), 4);
    }
}
