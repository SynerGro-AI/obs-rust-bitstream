//! Safe pure EncoderCaps (OBS encoder capability flags).
//!
//! Parsed from libobs/obs-encoder.h.
//! Flags: deprecated, pass_texture, dyn_bitrate, internal,
//! roi, scaling, multitrack_dyn_bitrate.
//! No unwraps.
//!
//! Differential: a caps with more flags reports more set.

/// Matching OBS_ENCODER_CAP_*.
pub const CAP_DEPRECATED: u32 = 1 << 0;
pub const CAP_PASS_TEXTURE: u32 = 1 << 1;
pub const CAP_DYN_BITRATE: u32 = 1 << 2;
pub const CAP_INTERNAL: u32 = 1 << 3;
pub const CAP_ROI: u32 = 1 << 4;
pub const CAP_SCALING: u32 = 1 << 5;
pub const CAP_MULTITRACK_DYN_BITRATE: u32 = 1 << 6;

#[derive(Clone, Debug, Default)]
pub struct EncoderCaps {
    flags: u32,
}

impl EncoderCaps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_flags(flags: u32) -> Self {
        Self { flags }
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn set(&mut self, flag: u32) {
        self.flags |= flag;
    }

    pub fn clear(&mut self, flag: u32) {
        self.flags &= !flag;
    }

    pub fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    /// Count of set capability bits.
    pub fn count(&self) -> u32 {
        self.flags.count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_has() {
        let mut c = EncoderCaps::new();
        c.set(CAP_ROI);
        c.set(CAP_SCALING);
        assert!(c.has(CAP_ROI));
        assert!(c.has(CAP_SCALING));
        assert!(!c.has(CAP_INTERNAL));
        assert_eq!(c.count(), 2);
    }

    #[test]
    fn clear() {
        let mut c = EncoderCaps::from_flags(CAP_DYN_BITRATE | CAP_ROI);
        c.clear(CAP_ROI);
        assert!(!c.has(CAP_ROI));
        assert!(c.has(CAP_DYN_BITRATE));
    }

    #[test]
    fn differential() {
        let mut a = EncoderCaps::new();
        let mut b = EncoderCaps::new();
        a.set(CAP_ROI);
        b.set(CAP_ROI);
        b.set(CAP_SCALING);
        b.set(CAP_PASS_TEXTURE);
        assert!(b.count() > a.count());
    }
}
