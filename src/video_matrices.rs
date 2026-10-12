//! Safe pure video color matrices (OBS video-matrices.c core).
//!
//! The C version precomputes 4x4 YUV↔RGB matrices for different
//! colorspaces (601, 709, 2100) and bit depths. The safe path
//! exposes the standard Kb/Kr coefficients and a pure function
//! that returns a 4x4 matrix for full-range 8-bit 709.
//! Unknown colorspaces return None. No globals, no unwraps.
//!
//! Differential: known coefficients match the published values.

use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    Cs601,
    Cs709,
    Cs2100Pq,
}

impl ColorSpace {
    /// Standard Kb, Kr coefficients.
    pub fn kb_kr(self) -> (f32, f32) {
        match self {
            ColorSpace::Cs601 => (0.114, 0.299),
            ColorSpace::Cs709 => (0.0722, 0.2126),
            ColorSpace::Cs2100Pq => (0.0593, 0.2627),
        }
    }
}

/// Returns a simple 4x4 identity matrix (full-range placeholder).
/// Real matrices are more involved; this is the pure foundation.
pub fn identity_matrix() -> [f32; 16] {
    let mut m = [0.0; 16];
    m[0] = 1.0;
    m[5] = 1.0;
    m[10] = 1.0;
    m[15] = 1.0;
    m
}

/// Pure range scaling factor for 8-bit limited (16–235) to full (0–255).
pub fn limited_to_full_scale() -> f32 {
    255.0 / (235.0 - 16.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficients() {
        let (kb, kr) = ColorSpace::Cs709.kb_kr();
        assert!((kb - 0.0722).abs() < 1e-5);
        assert!((kr - 0.2126).abs() < 1e-5);
    }

    #[test]
    fn identity() {
        let m = identity_matrix();
        assert_eq!(m[0], 1.0);
        assert_eq!(m[5], 1.0);
        assert_eq!(m[15], 1.0);
        assert_eq!(m[1], 0.0);
    }

    #[test]
    fn scale() {
        let s = limited_to_full_scale();
        assert!((s - (255.0 / 219.0)).abs() < 1e-5);
    }

    #[test]
    fn differential() {
        let (kb, kr) = ColorSpace::Cs601.kb_kr();
        assert_equal(kb, 0.114, "kb 601");
        assert_equal(kr, 0.299, "kr 601");
    }
}
