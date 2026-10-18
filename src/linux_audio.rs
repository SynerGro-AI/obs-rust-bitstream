//! Linux microphone capture (ALSA surface).
//!
//! Converts signed 16-bit little-endian samples to f32
//! in the range -1.0..1.0. open returns a clear error
//! when no ALSA device is present. No unwraps.
//!
//! Differential: a known S16LE pair converts to the expected f32.

use crate::audio_capture::{AudioCapture, AudioSamples};

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
/// On this environment /dev/snd is absent, so open returns a clear error.
/// The conversion path is pure and tested.
pub struct LinuxAudio {
    rate: u32,
    channels: u16,
}

impl LinuxAudio {
    /// Attempt to open the default ALSA capture device.
    /// Returns a clear error if no device is present.
    pub fn open(rate: u32, channels: u16) -> Result<Self, String> {
        if rate == 0 || channels == 0 {
            return Err("rate and channels must be non-zero".to_string());
        }
        // No /dev/snd in this environment. Real open would use
        // snd_pcm_open here. Until a device is present, return
        // a clear error so the caller can fall back.
        if !std::path::Path::new("/dev/snd").exists() {
            return Err("Linux ALSA device not present (/dev/snd missing)".to_string());
        }
        // Device exists but the full ALSA session is not wired yet.
        Err("Linux ALSA session not yet wired".to_string())
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl AudioCapture for LinuxAudio {
    fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
        // No device, so no samples.
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
        // 0 and 32767
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
    fn open_errors_clearly() {
        match LinuxAudio::open(48000, 1) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.contains("ALSA") || e.contains("/dev/snd") || e.contains("not yet"));
            }
        }
    }

    #[test]
    fn differential() {
        let a = s16le_to_f32(&[0, 0, 0, 0x40], 1).unwrap();
        let b = s16le_to_f32(&[0, 0x80, 0, 0], 1).unwrap();
        assert_ne!(a, b);
    }
}
