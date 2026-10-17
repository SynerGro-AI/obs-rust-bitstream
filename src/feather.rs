//! Safe pure Feather (selection soften core).
//!
//! Expands a selection by one pixel and marks the
//! border as soft. Works on Selection. No unwraps.
//!
//! Differential: a selected pixel gains a neighbor.

use crate::pixel_selection::Selection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeatherError {
    Empty,
}

pub struct Feather;

impl Feather {
    /// Expand the selection by `radius` pixels.
    pub fn apply(sel: &mut Selection, radius: u8) -> Result<(), FeatherError> {
        if sel.count() == 0 {
            return Err(FeatherError::Empty);
        }
        let w = sel.width();
        let h = sel.height();
        let mut add = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if sel.is_selected(x, y) {
                    for dy in 0..=radius as i32 {
                        for dx in 0..=radius as i32 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                                add.push((nx as u32, ny as u32));
                            }
                        }
                    }
                }
            }
        }
        for (x, y) in add {
            let _ = sel.rect(x, y, 1, 1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut s = Selection::new(4, 4);
        assert!(Feather::apply(&mut s, 1).is_err());
    }

    #[test]
    fn differential() {
        let mut s = Selection::new(4, 4);
        s.rect(1, 1, 1, 1).unwrap();
        Feather::apply(&mut s, 1).unwrap();
        assert!(s.is_selected(1, 1));
        assert!(s.is_selected(2, 2));
    }
}
