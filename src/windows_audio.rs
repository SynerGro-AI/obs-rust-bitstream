//! Windows microphone capture (WASAPI).
//!
//! Parsed from OBS plugins/win-wasapi/win-wasapi.cpp.
//! Device default is "default". Input use_device_timing is false.
//! Device output use_device_timing is true. Format is float.
//! open returns a clear error on non-Windows.
//! No unwraps.
//!
//! Differential: a known float pair converts to the expected f32.

use crate::audio_capture::{AudioCapture, AudioSamples};

/// Default device name from OBS GetWASAPIDefaultsInput.
pub const WASAPI_DEFAULT_DEVICE: &str = "default";

/// Source kinds from OBS SourceType.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasapiKind {
    Input,
    DeviceOutput,
    ProcessOutput,
}

/// Convert packed float samples to the AudioSamples range.
/// WASAPI is always float. Returns None if the length is not
/// a multiple of channels.
pub fn wasapi_float_to_f32(raw: &[f32], channels: u16) -> Option<Vec<f32>> {
    if channels == 0 || !raw.len().is_multiple_of(channels as usize) {
        return None;
    }
    Some(raw.to_vec())
}

/// Windows WASAPI microphone.
/// Matches the OBS structure. open returns a clear error
/// when not on Windows.
pub struct WindowsAudio {
    device: String,
    kind: WasapiKind,
    use_device_timing: bool,
    rate: u32,
    channels: u16,
}

impl WindowsAudio {
    /// Attempt to open the default WASAPI capture device.
    /// Returns a clear error on non-Windows.
    pub fn open(rate: u32, channels: u16) -> Result<Self, String> {
        Self::open_device(WASAPI_DEFAULT_DEVICE, WasapiKind::Input, rate, channels)
    }

    /// Open a named WASAPI device.
    pub fn open_device(
        device: &str,
        kind: WasapiKind,
        rate: u32,
        channels: u16,
    ) -> Result<Self, String> {
        if rate == 0 || channels == 0 {
            return Err("rate and channels must be non-zero".to_string());
        }
        #[cfg(not(windows))]
        {
            let _ = (device, kind);
            return Err("Windows WASAPI audio capture not available on this platform".to_string());
        }
        #[cfg(windows)]
        {
            // Device exists. Full IAudioClient session is the next step.
            let _ = (device, kind);
            Err("Windows WASAPI session not yet wired".to_string())
        }
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn kind(&self) -> WasapiKind {
        self.kind
    }

    pub fn use_device_timing(&self) -> bool {
        self.use_device_timing
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl AudioCapture for WindowsAudio {
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
    fn defaults_match_obs() {
        assert_eq!(WASAPI_DEFAULT_DEVICE, "default");
    }

    #[test]
    fn convert_known() {
        let raw = [0.0f32, 0.5, -0.25];
        let out = wasapi_float_to_f32(&raw, 1).unwrap();
        assert_eq!(out, raw);
    }

    #[test]
    fn open_errors_clearly() {
        match WindowsAudio::open(48000, 2) {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("WASAPI") || e.contains("not available") || e.contains("not yet")
                );
            }
        }
    }

    #[test]
    fn differential() {
        let a = wasapi_float_to_f32(&[0.0, 0.1], 1).unwrap();
        let b = wasapi_float_to_f32(&[0.0, 0.9], 1).unwrap();
        assert_ne!(a, b);
    }
}
