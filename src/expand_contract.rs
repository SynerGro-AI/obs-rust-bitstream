//! Safe pure ExpandContract (selection grow/shrink core).
//!
//! Expand grows the selection by one pixel. Contract
//! shrinks it by one pixel. Empty selection rejected
//! on expand. No unwraps.
//!
//! Differential: expand adds a neighbor, contract
//! removes an edge.

use crate::pixel_selection::Selection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpandError {
    Empty,
}

pub struct ExpandContract;

impl ExpandContract {
    /// Grow by one pixel in all directions.
    pub fn expand(sel: &mut Selection) -> Result<(), ExpandError> {
        if sel.count() == 0 {
            return Err(ExpandError::Empty);
        }
        let w = sel.width();
        let h = sel.height();
        let mut add = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if sel.is_selected(x, y) {
                    if x > 0 {
                        add.push((x - 1, y));
                    }
                    if x + 1 < w {
                        add.push((x + 1, y));
                    }
                    if y > 0 {
                        add.push((x, y - 1));
                    }
                    if y + 1 < h {
                        add.push((x, y + 1));
                    }
                }
            }
        }
        for (x, y) in add {
            let _ = sel.rect(x, y, 1, 1);
        }
        Ok(())
    }

    /// Shrink by one pixel. Edge pixels are cleared.
    pub fn contract(sel: &mut Selection) {
        let w = sel.width();
        let h = sel.height();
        let mut remove = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if sel.is_selected(x, y) {
                    let edge = (x == 0 || !sel.is_selected(x - 1, y))
                        || (x + 1 >= w || !sel.is_selected(x + 1, y))
                        || (y == 0 || !sel.is_selected(x, y - 1))
                        || (y + 1 >= h || !sel.is_selected(x, y + 1));
                    if edge {
                        remove.push((x, y));
                    }
                }
            }
        }
        // Clear by invert-select-invert is heavy; rebuild via a new mask.
        // We expose no clear, so we invert the removed set by a full rebuild.
        // For the pure path we mark them unselected by inverting twice around a temp.
        // Simpler: create a new selection and copy the keepers.
        let mut keep = Selection::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if sel.is_selected(x, y) && !remove.contains(&(x, y)) {
                    let _ = keep.rect(x, y, 1, 1);
                }
            }
        }
        *sel = keep;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut s = Selection::new(4, 4);
        assert!(ExpandContract::expand(&mut s).is_err());
    }

    #[test]
    fn differential() {
        let mut s = Selection::new(4, 4);
        s.rect(1, 1, 1, 1).unwrap();
        ExpandContract::expand(&mut s).unwrap();
        assert!(s.is_selected(0, 1));
        let before = s.count();
        ExpandContract::contract(&mut s);
        assert!(s.count() < before);
    }
}
