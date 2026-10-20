//! Safe pure audio mixer foundation.
//!
//! Simple per-sample gain and two-buffer sum. Zero length and
//! mismatched lengths return None. No unwraps, no panics.
//!
//! Differential: sum of identical buffers is 2x.

#[cfg(test)]
use crate::diff::assert_equal;

/// Apply a linear gain to every sample (f32).
pub fn apply_gain(samples: &[f32], gain: f32) -> Vec<f32> {
    samples.iter().map(|s| s * gain).collect()
}

/// Sum two equal-length f32 buffers. Returns None if lengths differ.
pub fn sum_buffers(a: &[f32], b: &[f32]) -> Option<Vec<f32>> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    Some(a.iter().zip(b.iter()).map(|(x, y)| x + y).collect())
}

/// Peak absolute value (or 0.0 for empty).
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain() {
        let out = apply_gain(&[1.0, -0.5], 2.0);
        assert_eq!(out, vec![2.0, -1.0]);
    }

    #[test]
    fn sum() {
        let out = sum_buffers(&[1.0, 2.0], &[3.0, 4.0]).unwrap();
        assert_eq!(out, vec![4.0, 6.0]);
    }

    #[test]
    fn mismatch_safe() {
        assert!(sum_buffers(&[1.0], &[1.0, 2.0]).is_none());
        assert!(sum_buffers(&[], &[]).is_none());
    }

    #[test]
    fn differential() {
        let a = [0.5f32, 0.5];
        let out = sum_buffers(&a, &a).unwrap();
        assert_equal(out[0], 1.0, "sum identical");
    }
}
