//! Safe pure Selection (marquee core).
//!
//! A selection owns a name and a rectangle (x, y, w, h).
//! Empty names rejected. No unwraps.
//!
//! Differential: set bounds + contains round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Selection {
    name: String,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Selection {
    pub fn new(name: &str) -> Result<Self, SelectionError> {
        if name.is_empty() {
            return Err(SelectionError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_bounds(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.w = w.max(0.0);
        self.h = h.max(0.0);
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    pub fn area(&self) -> f32 {
        self.w * self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = Selection::new("marquee").unwrap();
        assert_eq!(s.area(), 0.0);
    }

    #[test]
    fn empty_name() {
        assert!(Selection::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = Selection::new("marquee").unwrap();
        s.set_bounds(10.0, 20.0, 100.0, 50.0);
        assert!(s.contains(50.0, 40.0));
        assert!(!s.contains(0.0, 0.0));
        assert_eq!(s.area(), 5000.0);
    }
}
