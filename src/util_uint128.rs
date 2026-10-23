//! Minimal 128-bit helpers (OBS util_uint128.h core).
//!
//! Safe add and mul64_64 using native u128. The C version manually
//! splits into 32-bit limbs; the math is identical when no carry is lost.
//!
//! Differential: our results match a direct u128 calculation.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct U128 {
    pub low: u64,
    pub high: u64,
}

impl U128 {
    pub fn from_u128(v: u128) -> Self {
        Self {
            low: v as u64,
            high: (v >> 64) as u64,
        }
    }
    pub fn to_u128(self) -> u128 {
        (self.high as u128) << 64 | self.low as u128
    }
}

pub fn util_add128(a: U128, b: U128) -> U128 {
    U128::from_u128(a.to_u128().wrapping_add(b.to_u128()))
}

pub fn util_mul64_64(a: u64, b: u64) -> U128 {
    U128::from_u128((a as u128) * (b as u128))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let a = U128 { low: 1, high: 0 };
        let b = U128 {
            low: u64::MAX,
            high: 0,
        };
        let s = util_add128(a, b);
        assert_eq!(s.low, 0);
        assert_eq!(s.high, 1);
    }

    #[test]
    fn mul() {
        let r = util_mul64_64(0x1_0000_0000, 0x1_0000_0000);
        assert_eq!(r.low, 0);
        assert_eq!(r.high, 1);
    }

    #[test]
    fn differential() {
        let a = U128::from_u128(42);
        let b = U128::from_u128(100);
        let safe = util_add128(a, b);
        let expected = U128::from_u128(142);
        assert_equal(safe, expected, "add128");
    }
}
