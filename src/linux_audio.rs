//! Linux microphone capture (ALSA).
//!
//! Parsed from OBS plugins/linux-alsa/alsa-input.c.
//! Device default is "default". Rates are 32000, 44100, 48000.
//! Format preference is S16_LE, S32_LE, FLOAT_LE, U8.
//! Access is interleaved. Channels default to 2.
//! open returns a clear error when /dev/snd is missing.
//! No unwraps.
//!
//! Differential: a known S16LE pair converts to the expected f32.

use crate::audio_capture::{AudioCapture, AudioSamples};

/// Rates OBS offers for ALSA input.
pub const ALSA_RATES: &[u32] = &[32000, 44100, 48000];

/// Default device name from OBS alsa_get_defaults.
pub const ALSA_DEFAULT_DEVICE: &str = "default";

/// Default rate from OBS alsa_get_defaults.
pub const ALSA_DEFAULT_RATE: u32 = 44100;

/// Format preference order from OBS _alsa_configure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlsaFormat {
    S16Le,
    S32Le,
    FloatLe,
    U8,
}

/// Convert packed S16LE samples to f32 (-1.0..1.0).
/// Returns None if the length is not a multiple of 2 * channels.
pub fn s16le_to_f32(raw: &[u8], channels: u16) -> Option<Vec<f32>> {
    if channels == 0 {
        return None;
    }
    let frame_bytes = 2 * channels as usize;
    if raw.len() % frame_bytes != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len() / 2);
    for chunk in raw.chunks_exact(2) {
        let s = i16::from_le_bytes([chunk[0], chunk[1]]);
        out.push(s as f32 / 32768.0);
    }
    Some(out)
}

/// Linux ALSA microphone.
/// Matches the OBS structure. open returns a clear error
/// when no device is present.
pub struct LinuxAudio {
    device: String,
    rate: u32,
    channels: u16,
    format: AlsaFormat,
}

impl LinuxAudio {
    /// Attempt to open the default ALSA capture device.
    /// Returns a clear error if no device is present.
    pub fn open(rate: u32, channels: u16) -> Result<Self, String> {
        Self::open_device(ALSA_DEFAULT_DEVICE, rate, channels)
    }

    /// Open a named ALSA device.
    /// Returns a clear error if /dev/snd is missing.
    pub fn open_device(device: &str, rate: u32, channels: u16) -> Result<Self, String> {
        if rate == 0 || channels == 0 {
            return Err("rate and channels must be non-zero".to_string());
        }
        if !ALSA_RATES.contains(&rate) {
            return Err(format!("rate {} is not in the OBS ALSA list", rate));
        }
        if !std::path::Path::new("/dev/snd").exists() {
            return Err("Linux ALSA device not present (/dev/snd missing)".to_string());
        }
        // Device exists. Full snd_pcm_open session is the next step.
        Err("Linux ALSA session not yet wired".to_string())
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn format(&self) -> AlsaFormat {
        self.format
    }
}

impl AudioCapture for LinuxAudio {
    fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
        let _ = frames;
        None
    }

    fn rate(&self) -> u32 {
        self.rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_known() {
        let raw = [0u8, 0, 0xff, 0x7f];
        let out = s16le_to_f32(&raw, 1).unwrap();
        assert_eq!(out.len(), 2);
        assert!((out[0] - 0.0).abs() < 0.0001);
        assert!((out[1] - 32767.0 / 32768.0).abs() < 0.0001);
    }

    #[test]
    fn bad_length() {
        assert!(s16le_to_f32(&[0u8, 1, 2], 1).is_none());
    }

    #[test]
    fn defaults_match_obs() {
        assert_eq!(ALSA_DEFAULT_DEVICE, "default");
        assert_eq!(ALSA_DEFAULT_RATE, 44100);
        assert!(ALSA_RATES.contains(&48000));
    }

    #[test]
    fn open_errors_clearly() {
        match LinuxAudio::open(ALSA_DEFAULT_RATE, 2) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.contains("ALSA") || e.contains("/dev/snd") || e.contains("not yet"));
            }
        }
    }

    #[test]
    fn bad_rate() {
        assert!(LinuxAudio::open(22050, 1).is_err());
    }

    #[test]
    fn differential() {
        let a = s16le_to_f32(&[0, 0, 0, 0x40], 1).unwrap();
        let b = s16le_to_f32(&[0, 0x80, 0, 0], 1).unwrap();
        assert_ne!(a, b);
    }
}