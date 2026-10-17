//! Safe pure Fluid (grid density core).
//!
//! Holds a density grid. Step diffuses density to
//! neighbors. No unwraps.
//!
//! Differential: a high cell spreads to a neighbor.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug)]
pub struct Fluid {
    width: usize,
    height: usize,
    density: Vec<f32>,
    diffuse: f32,
}

impl Fluid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            density: vec![0.0; width * height],
            diffuse: 0.1,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.density[y * self.width + x] = value;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.density[y * self.width + x]
        } else {
            0.0
        }
    }

    /// One diffusion step.
    pub fn step(&mut self) {
        let mut next = self.density.clone();
        for y in 1..self.height.saturating_sub(1) {
            for x in 1..self.width.saturating_sub(1) {
                let i = y * self.width + x;
                let avg = (self.density[i - 1]
                    + self.density[i + 1]
                    + self.density[i - self.width]
                    + self.density[i + self.width])
                    * 0.25;
                next[i] = self.density[i] + (avg - self.density[i]) * self.diffuse;
            }
        }
        self.density = next;
    }

    /// Read diffuse rate from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(d) = weights.get("fluid.diffuse") {
            self.diffuse = d.clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = Fluid::new(4, 4);
        f.set(1, 1, 100.0);
        f.step();
        assert!(f.get(1, 2) > 0.0);
        let mut w = LiveWeights::new();
        w.set("fluid.diffuse", 0.0).unwrap();
        f.from_weights(&w);
        assert_eq!(f.diffuse, 0.0);
    }
}
