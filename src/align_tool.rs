//! Safe pure AlignTool (align and distribute).
//!
//! Aligns selected items to an edge or center.
//! Distribute spaces them evenly. No unwraps.
//!
//! Differential: a different align produces a different layout.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

pub struct AlignTool;

impl AlignTool {
    /// Align positions to an edge of the canvas. Returns the new positions.
    pub fn align(positions: &[(f32, f32)], canvas_w: f32, canvas_h: f32, how: Align) -> Vec<(f32, f32)> {
        positions
            .iter()
            .map(|&(x, y)| match how {
                Align::Left => (0.0, y),
                Align::Center => (canvas_w / 2.0, y),
                Align::Right => (canvas_w, y),
                Align::Top => (x, 0.0),
                Align::Middle => (x, canvas_h / 2.0),
                Align::Bottom => (x, canvas_h),
            })
            .collect()
    }

    /// Distribute positions evenly between the first and last.
    pub fn distribute(positions: &[(f32, f32)]) -> Vec<(f32, f32)> {
        if positions.len() < 3 {
            return positions.to_vec();
        }
        let first = positions[0];
        let last = positions[positions.len() - 1];
        let n = positions.len() as f32;
        positions
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let t = i as f32 / (n - 1.0);
                (first.0 + (last.0 - first.0) * t, first.1 + (last.1 - first.1) * t)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_left() {
        let pos = [(10.0, 5.0), (20.0, 15.0)];
        let out = AlignTool::align(&pos, 100.0, 100.0, Align::Left);
        assert_eq!(out[0].0, 0.0);
        assert_eq!(out[1].0, 0.0);
    }

    #[test]
    fn distribute() {
        let pos = [(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)];
        let out = AlignTool::distribute(&pos);
        assert!((out[1].0 - 5.0).abs() < 0.01);
    }

    #[test]
    fn differential() {
        let pos = [(10.0, 10.0)];
        let a = AlignTool::align(&pos, 100.0, 100.0, Align::Left);
        let b = AlignTool::align(&pos, 100.0, 100.0, Align::Right);
        assert_ne!(a[0].0, b[0].0);
    }
}
