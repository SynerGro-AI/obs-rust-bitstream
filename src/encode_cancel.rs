//! Safe pure EncodeCancel (stop an encode core).
//!
//! A flag that can be set to stop an encode.
//! No unwraps.
//!
//! Differential: cancel stops further ticks.

#[derive(Clone, Debug, Default)]
pub struct EncodeCancel {
    cancelled: bool,
}

impl EncodeCancel {
    pub fn new() -> Self {
        Self { cancelled: false }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn reset(&mut self) {
        self.cancelled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut c = EncodeCancel::new();
        assert!(!c.is_cancelled());
        c.cancel();
        assert!(c.is_cancelled());
        c.reset();
        assert!(!c.is_cancelled());
    }
}
