//! Safe 64-bit multiply-then-divide (OBS util_uint64.h).
//!
//! The C version uses either _umul128/_udiv128 or a rem-based reduction
//! that still can overflow the intermediate (rem * mul). The safe path
//! uses a 128-bit intermediate (via u128) so the math is exact for all
//! inputs that fit in the result.
//!
//! Differential: safe result equals the mathematical (num * mul) / div
//! whenever the product fits in u128.

use crate::diff::assert_equal;

/// Exact (num * mul) / div using 128-bit intermediate.
/// Returns 0 if div == 0 (C behavior is undefined; we choose a safe zero).
pub fn util_mul_div64(num: u64, mul: u64, div: u64) -> u64 {
    if div == 0 {
        return 0;
    }
    let product = (num as u128) * (mul as u128);
    (product / div as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(util_mul_div64(10, 20, 5), 40);
        assert_eq!(util_mul_div64(0, 100, 1), 0);
        assert_eq!(util_mul_div64(100, 0, 1), 0);
    }

    #[test]
    fn div_zero_safe() {
        assert_eq!(util_mul_div64(5, 5, 0), 0);
    }

    #[test]
    fn large_no_overflow() {
        // 0xFFFFFFFF * 0xFFFFFFFF / 1 should not wrap in the intermediate
        let n = 0xFFFF_FFFFu64;
        assert_eq!(util_mul_div64(n, n, 1), n.saturating_mul(n));
    }

    #[test]
    fn differential() {
        let cases = [(10, 20, 5), (100, 3, 7), (0, 1, 1), (1, 1, 1)];
        for (n, m, d) in cases {
            let safe = util_mul_div64(n, m, d);
            let expected = ((n as u128 * m as u128) / d as u128) as u64;
            assert_equal(safe, expected, "mul_div64");
        }
    }
}
