//! Safe pure EncoderSelect (hardware or software core).
//!
//! Picks a hardware encoder when a GPU is present
//! and software otherwise. No unwraps.
//!
//! Differential: no GPU selects software.

use crate::gpu_detect::{GpuDetect, GpuKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EncoderChoice {
    Software,
    Nvidia,
    Amd,
    Intel,
}

impl EncoderChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncoderChoice::Software => "software",
            EncoderChoice::Nvidia => "nvenc",
            EncoderChoice::Amd => "amf",
            EncoderChoice::Intel => "qsv",
        }
    }
}

pub struct EncoderSelect;

impl EncoderSelect {
    /// Pick an encoder from the detected GPU.
    pub fn choose() -> EncoderChoice {
        let info = GpuDetect::detect();
        if !info.is_present() {
            return EncoderChoice::Software;
        }
        match info.kind {
            GpuKind::Nvidia => EncoderChoice::Nvidia,
            GpuKind::Amd => EncoderChoice::Amd,
            GpuKind::Intel => EncoderChoice::Intel,
            _ => EncoderChoice::Software,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let choice = EncoderSelect::choose();
        // No GPU here
        assert_eq!(choice, EncoderChoice::Software);
        assert_eq!(choice.as_str(), "software");
    }
}
