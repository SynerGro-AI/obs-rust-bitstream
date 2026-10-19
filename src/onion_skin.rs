//! Safe pure OnionSkin (ToonBoom onion skin tool).
//!
//! Holds the number of frames before and after the current
//! frame to show as onion skins. Set the range or clear.
//! No unwraps.
//!
//! Differential: a different range produces a different skin.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnionSkin {
    before: u32,
    after: u32,
    enabled: bool,
}

impl OnionSkin {
    pub fn new() -> Self {
        Self {
            before: 1,
            after: 1,
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn set_range(&mut self, before: u32, after: u32) {
        self.before = before.min(5);
        self.after = after.min(5);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn before(&self) -> u32 {
        self.before
    }

    pub fn after(&self) -> u32 {
        self.after
    }

    /// Total frames shown (before + current + after) when enabled.
    pub fn frame_count(&self) -> u32 {
        if self.enabled {
            self.before + 1 + self.after
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enables() {
        let mut o = OnionSkin::new();
        o.enable();
        o.set_range(2, 1);
        assert!(o.is_enabled());
        assert_eq!(o.frame_count(), 4);
    }

    #[test]
    fn disabled_is_one() {
        let o = OnionSkin::new();
        assert_eq!(o.frame_count(), 1);
    }

    #[test]
    fn differential() {
        let mut a = OnionSkin::new();
        a.enable();
        a.set_range(1, 1);
        let mut b = OnionSkin::new();
        b.enable();
        b.set_range(3, 2);
        assert_ne!(a.frame_count(), b.frame_count());
    }
}
