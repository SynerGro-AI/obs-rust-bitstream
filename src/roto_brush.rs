//! Safe pure RotoBrush (edge selection tool).
//!
//! Holds a list of edge points that define a roto selection.
//! Add a point, clear, or check if closed. No unwraps.
//!
//! Differential: more points produce a longer edge.

#[derive(Clone, Debug, PartialEq)]
pub struct RotoPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default)]
pub struct RotoBrush {
    edge: Vec<RotoPoint>,
}

impl RotoBrush {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: f32, y: f32) {
        self.edge.push(RotoPoint { x, y });
    }

    pub fn clear(&mut self) {
        self.edge.clear();
    }

    pub fn len(&self) -> usize {
        self.edge.len()
    }

    pub fn is_closed(&self) -> bool {
        self.edge.len() >= 3
    }

    pub fn edge(&self) -> &[RotoPoint] {
        &self.edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        let mut r = RotoBrush::new();
        r.add(0.0, 0.0);
        r.add(10.0, 0.0);
        r.add(5.0, 10.0);
        assert!(r.is_closed());
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn not_closed() {
        let mut r = RotoBrush::new();
        r.add(0.0, 0.0);
        assert!(!r.is_closed());
    }

    #[test]
    fn differential() {
        let mut a = RotoBrush::new();
        a.add(0.0, 0.0);
        let mut b = RotoBrush::new();
        b.add(0.0, 0.0);
        b.add(1.0, 1.0);
        assert_ne!(a.len(), b.len());
    }
}
