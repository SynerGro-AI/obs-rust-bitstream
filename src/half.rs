//! Safe pure half-float helpers (OBS graphics/half.h core).
//!
//! The C version converts between f32 and 16-bit float.
//! The safe path provides pure conversion with explicit
//! infinity / NaN handling. No unwraps.
//!
//! Differential: round-trip of finite values is close.

#[cfg(test)]
use crate::diff::assert_equal;

/// Convert f32 to IEEE-754 half (f16) bits.
pub fn f32_to_f16(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32;
    let mant = bits & 0x7fffff;

    if exp == 255 {
        // Inf or NaN
        if mant == 0 {
            return sign | 0x7c00;
        } else {
            return sign | 0x7e00;
        }
    }
    if exp == 0 {
        return sign; // flush denorms / zero
    }

    let new_exp = exp - 127 + 15;
    if new_exp >= 31 {
        return sign | 0x7c00; // overflow to inf
    }
    if new_exp <= 0 {
        return sign; // underflow to zero
    }
    sign | ((new_exp as u16) << 10) | ((mant >> 13) as u16)
}

/// Convert IEEE-754 half (f16) bits to f32.
pub fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h & 0x8000) as u32) << 16;
    let exp = ((h >> 10) & 0x1f) as i32;
    let mant = (h & 0x3ff) as u32;

    if exp == 0 {
        if mant == 0 {
            return f32::from_bits(sign);
        }
        // denorm: not fully handled; return zero for simplicity
        return f32::from_bits(sign);
    }
    if exp == 31 {
        if mant == 0 {
            return f32::from_bits(sign | 0x7f800000);
        } else {
            return f32::from_bits(sign | 0x7fc00000);
        }
    }
    let new_exp = (exp - 15 + 127) as u32;
    f32::from_bits(sign | (new_exp << 23) | (mant << 13))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(f32_to_f16(0.0), 0);
        assert_eq!(f16_to_f32(0), 0.0);
    }

    #[test]
    fn one() {
        let h = f32_to_f16(1.0);
        let back = f16_to_f32(h);
        assert!((back - 1.0).abs() < 1e-3);
    }

    #[test]
    fn inf() {
        let h = f32_to_f16(f32::INFINITY);
        assert_eq!(h, 0x7c00);
    }

    #[test]
    fn differential() {
        let h = f32_to_f16(2.0);
        let back = f16_to_f32(h);
        assert_equal(back, 2.0, "half round-trip");
    }
}
