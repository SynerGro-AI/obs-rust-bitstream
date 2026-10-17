//! Safe pure Lasso (polygon selection core).
//!
//! Selects pixels inside a closed polygon using
//! even-odd fill. Empty polygon rejected. No unwraps.
//!
//! Differential: a point inside a square is selected.

use crate::pixel_selection::Selection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LassoError {
    Empty,
    SizeMismatch,
}

pub struct Lasso;

impl Lasso {
    /// points are (x, y) pairs. The polygon is closed
    /// automatically. Selects into an existing Selection.
    pub fn select(sel: &mut Selection, points: &[(u32, u32)]) -> Result<(), LassoError> {
        if points.len() < 3 {
            return Err(LassoError::Empty);
        }
        let w = sel.width();
        let h = sel.height();
        for y in 0..h {
            for x in 0..w {
                if inside(x, y, points) {
                    let _ = sel.rect(x, y, 1, 1);
                }
            }
        }
        Ok(())
    }
}

fn inside(x: u32, y: u32, points: &[(u32, u32)]) -> bool {
    let mut c = false;
    let n = points.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if ((yi > y) != (yj > y))
            && (x < (xj as i32 - xi as i32) * (y as i32 - yi as i32) / (yj as i32 - yi as i32) as u32 + xi)
        {
            c = !c;
        }
        j = i;
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut s = Selection::new(8, 8);
        assert!(Lasso::select(&mut s, &[(0, 0), (1, 1)]).is_err());
    }

    #[test]
    fn differential() {
        let mut s = Selection::new(8, 8);
        let pts = [(1, 1), (5, 1), (5, 5), (1, 5)];
        Lasso::select(&mut s, &pts).unwrap();
        assert!(s.is_selected(3, 3));
        assert!(!s.is_selected(0, 0));
    }
}
