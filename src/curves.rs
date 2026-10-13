//! Safe pure Curves (tone curve core).
//!
//! A curve owns control points and remaps a 0-1 value by
//! linear interpolation. No unwraps.
//!
//! Differential: a point at (0.5, 0.8) raises 0.5 to 0.8.

#[derive(Clone, Debug)]
pub struct Curves {
    points: Vec<(f32, f32)>,
}

impl Curves {
    pub fn new() -> Self {
        Self {
            points: vec![(0.0, 0.0), (1.0, 1.0)],
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32) {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        self.points.push((x, y));
        self.points
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Remap a 0-1 value through the curve.
    pub fn apply(&self, v: f32) -> f32 {
        let v = v.clamp(0.0, 1.0);
        if self.points.is_empty() {
            return v;
        }
        let mut prev = self.points[0];
        for p in &self.points {
            if v <= p.0 {
                let span = (p.0 - prev.0).max(0.0001);
                let t = (v - prev.0) / span;
                return prev.1 + t * (p.1 - prev.1);
            }
            prev = *p;
        }
        self.points.last().map(|p| p.1).unwrap_or(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let c = Curves::new();
        assert!((c.apply(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let mut c = Curves::new();
        c.add_point(0.5, 0.8);
        assert!((c.apply(0.5) - 0.8).abs() < 0.001);
        assert!(c.apply(0.25) > 0.25);
    }
}
