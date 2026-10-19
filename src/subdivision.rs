//! Safe pure Subdivision (mesh level tool).
//!
//! Holds a subdivision level. Increase, decrease, or set.
//! Level is clamped to 0..=5. No unwraps.
//!
//! Differential: a different level produces a different mesh density.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subdivision {
    level: u8,
}

impl Subdivision {
    pub fn new() -> Self {
        Self { level: 0 }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn set(&mut self, level: u8) {
        self.level = level.min(5);
    }

    pub fn increase(&mut self) {
        if self.level < 5 {
            self.level += 1;
        }
    }

    pub fn decrease(&mut self) {
        if self.level > 0 {
            self.level -= 1;
        }
    }

    /// Face count multiplier. Level 0 is 1x. Each level quadruples.
    pub fn density(&self) -> u32 {
        1u32 << (self.level * 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increases() {
        let mut s = Subdivision::new();
        s.increase();
        s.increase();
        assert_eq!(s.level(), 2);
        assert_eq!(s.density(), 16);
    }

    #[test]
    fn clamps() {
        let mut s = Subdivision::new();
        s.set(10);
        assert_eq!(s.level(), 5);
    }

    #[test]
    fn differential() {
        let mut a = Subdivision::new();
        let mut b = Subdivision::new();
        a.set(1);
        b.set(3);
        assert_ne!(a.density(), b.density());
    }
}
