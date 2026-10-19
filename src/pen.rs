//! Safe pure Pen (path tool).
//!
//! Holds a list of anchor points. Add a point, close the path,
//! or clear. Empty paths are rejected on close. No unwraps.
//!
//! Differential: more points produce a longer path.

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Pen {
    points: Vec<Point>,
    closed: bool,
}

impl Pen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an anchor point.
    pub fn add(&mut self, x: f32, y: f32) {
        self.points.push(Point { x, y });
        self.closed = false;
    }

    /// Close the path. Returns false if fewer than 3 points.
    pub fn close(&mut self) -> bool {
        if self.points.len() < 3 {
            return false;
        }
        self.closed = true;
        true
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.closed = false;
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_close() {
        let mut p = Pen::new();
        p.add(0.0, 0.0);
        p.add(10.0, 0.0);
        p.add(5.0, 10.0);
        assert!(p.close());
        assert!(p.is_closed());
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn close_rejected() {
        let mut p = Pen::new();
        p.add(0.0, 0.0);
        assert!(!p.close());
    }

    #[test]
    fn differential() {
        let mut a = Pen::new();
        a.add(0.0, 0.0);
        let mut b = Pen::new();
        b.add(0.0, 0.0);
        b.add(1.0, 1.0);
        assert_ne!(a.len(), b.len());
    }
}
