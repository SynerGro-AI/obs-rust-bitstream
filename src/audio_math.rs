//! Safe pure audio math (OBS audio-math.h).
//!
//! The C version converts between linear multiplier and dB.
//! The safe path returns -inf for zero multiplier and 0.0 for
//! non-finite dB. No unwraps, no panics.
//!
//! Differential: round-trip mul → db → mul is close to original
//! for normal values.

use crate::diff::assert_equal;

pub fn mul_to_db(mul: f32) -> f32 {
    if mul <= 0.0 {
        f32::NEG_INFINITY
    } else {
        20.0 * mul.log10()
    }
}

pub fn db_to_mul(db: f32) -> f32 {
    if !db.is_finite() {
        0.0
    } else {
        10.0f32.powf(db / 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_mul() {
        assert_eq!(mul_to_db(0.0), f32::NEG_INFINITY);
    }

    #[test]
    fn unity() {
        assert!((mul_to_db(1.0) - 0.0).abs() < 1e-5);
        assert!((db_to_mul(0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn non_finite_safe() {
        assert_eq!(db_to_mul(f32::NAN), 0.0);
        assert_eq!(db_to_mul(f32::INFINITY), 0.0);
    }

    #[test]
    fn differential() {
        let mul = 0.5f32;
        let db = mul_to_db(mul);
        let back = db_to_mul(db);
        assert_equal((back - mul).abs() < 1e-5, true, "audio round-trip");
    }
}
