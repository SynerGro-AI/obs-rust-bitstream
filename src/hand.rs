//! Safe pure Hand (pan offset core).
//!
//! Holds a pan offset and moves it by deltas.
//! No unwraps.
//!
//! Differential: pan right + down round-trips.

#[derive(Clone, Debug)]
pub struct Hand {
    x: f32,
    y: f32,
}

impl Hand {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn offset(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut h = Hand::new();
        h.pan(10.0, 20.0);
        assert_eq!(h.offset(), (10.0, 20.0));
        h.pan(-5.0, -5.0);
        assert_eq!(h.offset(), (5.0, 15.0));
        h.reset();
        assert_eq!(h.offset(), (0.0, 0.0));
    }
}
