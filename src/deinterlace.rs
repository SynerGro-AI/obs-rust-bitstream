//! Safe pure Deinterlace (field weave core).
//!
//! Weaves two fields into one frame. Odd lines from the
//! first field, even from the second. No unwraps.
//!
//! Differential: two 2-row fields weave into 4 rows.

pub struct Deinterlace;

impl Deinterlace {
    /// Weave top and bottom fields. Each field is half the height.
    pub fn weave(top: &[u8], bottom: &[u8], width: usize) -> Vec<u8> {
        if width == 0 || top.len() != bottom.len() || !top.len().is_multiple_of(width) {
            return Vec::new();
        }
        let rows = top.len() / width;
        let mut out = Vec::with_capacity(top.len() + bottom.len());
        for r in 0..rows {
            let start = r * width;
            let end = start + width;
            out.extend_from_slice(&top[start..end]);
            out.extend_from_slice(&bottom[start..end]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(Deinterlace::weave(&[], &[], 0).is_empty());
    }

    #[test]
    fn differential() {
        let top = [1, 2, 3, 4];
        let bottom = [5, 6, 7, 8];
        let out = Deinterlace::weave(&top, &bottom, 2);
        assert_eq!(out, vec![1, 2, 5, 6, 3, 4, 7, 8]);
    }
}
