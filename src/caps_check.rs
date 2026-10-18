//! Safe pure CapsCheck (encoder capability gate).
//!
//! Checks whether an EncoderCaps supports a requested flag.
//! Returns true only if the flag is set. No unwraps.
//!
//! Differential: a caps with the flag passes; one without fails.

use crate::encoder_caps::EncoderCaps;

pub struct CapsCheck;

impl CapsCheck {
    /// True if the caps include the requested flag.
    pub fn supports(caps: &EncoderCaps, flag: u32) -> bool {
        caps.has(flag)
    }

    /// True if the caps include every flag in the list.
    pub fn supports_all(caps: &EncoderCaps, flags: &[u32]) -> bool {
        flags.iter().all(|f| caps.has(*f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_caps::{EncoderCaps, CAP_ROI, CAP_SCALING, CAP_INTERNAL};

    #[test]
    fn supports_one() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        assert!(CapsCheck::supports(&c, CAP_ROI));
        assert!(!CapsCheck::supports(&c, CAP_SCALING));
    }

    #[test]
    fn supports_all() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        c.set(CAP_SCALING);
        assert!(CapsCheck::supports_all(&c, &[CAP_ROI, CAP_SCALING]));
        assert!(!CapsCheck::supports_all(&c, &[CAP_ROI, CAP_INTERNAL]));
    }

    #[test]
    fn differential() {
        let mut a = EncoderCaps::new();
        let mut b = EncoderCaps::new();
        a.set(CAP_ROI);
        b.set(CAP_ROI);
        b.set(CAP_SCALING);
        assert!(CapsCheck::supports_all(&b, &[CAP_ROI, CAP_SCALING]));
        assert!(!CapsCheck::supports_all(&a, &[CAP_ROI, CAP_SCALING]));
    }
}
