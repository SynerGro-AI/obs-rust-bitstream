//! Safe pure WeightInterp (weight over frames core).
//!
//! Returns a weight for a frame index, interpolated
//! between start and end. No unwraps.
//!
//! Differential: frame 0 is start, last frame is end.

use crate::live_weights::LiveWeights;

pub struct WeightInterp;

impl WeightInterp {
    /// Interpolate a named weight from start to end across count frames.
    pub fn set(
        weights: &mut LiveWeights,
        name: &str,
        index: usize,
        count: usize,
        start: f32,
        end: f32,
    ) {
        if count <= 1 {
            let _ = weights.set(name, start);
            return;
        }
        let t = index as f32 / (count - 1) as f32;
        let v = start + (end - start) * t;
        let _ = weights.set(name, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut w = LiveWeights::new();
        WeightInterp::set(&mut w, "pipe.bright", 0, 4, 0.0, 30.0);
        assert_eq!(w.get("pipe.bright").unwrap(), 0.0);
        WeightInterp::set(&mut w, "pipe.bright", 3, 4, 0.0, 30.0);
        assert_eq!(w.get("pipe.bright").unwrap(), 30.0);
        WeightInterp::set(&mut w, "pipe.bright", 1, 4, 0.0, 30.0);
        assert!((w.get("pipe.bright").unwrap() - 10.0).abs() < 0.1);
    }
}
