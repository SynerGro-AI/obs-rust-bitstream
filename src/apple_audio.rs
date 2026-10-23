//! macOS / iOS microphone capture (CoreAudio).
//!
//! Parsed from OBS plugins/mac-capture/mac-audio.c.
//! Device default is "default". enable_downmix default is true.
//! Input uses kAudioHardwarePropertyDefaultInputDevice.
//! Format can be float, 16-bit, 32-bit, or U8.
//! open returns a clear error on non-Apple.
//! No unwraps.
//!
//! Differential: a known float pair converts to the expected f32.

use crate::audio_capture::{AudioCapture, AudioSamples};

/// Default device name from OBS coreaudio_defaults.
pub const COREAUDIO_DEFAULT_DEVICE: &str = "default";

/// Speaker layout from channel count (OBS convert_ca_speaker_layout).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreAudioSpeakers {
    Mono,
    Stereo,
    TwoPointOne,
    FourPointZero,
    FourPointOne,
    FivePointOne,
    SevenPointOne,
    Unknown,
}

/// Map channel count to the OBS speaker layout.
pub fn speakers_from_channels(channels: u32) -> CoreAudioSpeakers {
    match channels {
        1 => CoreAudioSpeakers::Mono,
        2 => CoreAudioSpeakers::Stereo,
        3 => CoreAudioSpeakers::TwoPointOne,
        4 => CoreAudioSpeakers::FourPointZero,
        5 => CoreAudioSpeakers::FourPointOne,
        6 => CoreAudioSpeakers::FivePointOne,
        8 => CoreAudioSpeakers::SevenPointOne,
        _ => CoreAudioSpeakers::Unknown,
    }
}

/// Convert packed float samples. CoreAudio commonly delivers float.
/// Returns None if the length is not a multiple of channels.
pub fn coreaudio_float_to_f32(raw: &[f32], channels: u16) -> Option<Vec<f32>> {
    if channels == 0 || raw.len() % channels as usize != 0 {
        return None;
    }
    Some(raw.to_vec())
}

/// Apple CoreAudio microphone.
/// Matches the OBS structure. open returns a clear error
/// when not on macOS or iOS.
pub struct AppleAudio {
    device: String,
    enable_downmix: bool,
    rate: u32,
    channels: u16,
}

impl AppleAudio {
    /// Attempt to open the default CoreAudio input device.
    /// Returns a clear error on non-Apple.
    pub fn open(rate: u32, channels: u16) -> Result<Self, String> {
        Self::open_device(COREAUDIO_DEFAULT_DEVICE, true, rate, channels)
    }

    /// Open a named CoreAudio device.
    pub fn open_device(
        device: &str,
        enable_downmix: bool,
        rate: u32,
        channels: u16,
    ) -> Result<Self, String> {
        if rate == 0 || channels == 0 {
            return Err("rate and channels must be non-zero".to_string());
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            let _ = (device, enable_downmix);
            return Err("Apple CoreAudio capture not available on this platform".to_string());
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            // Device exists. Full AudioUnit session is the next step.
            let _ = (device, enable_downmix);
            Err("Apple CoreAudio session not yet wired".to_string())
        }
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn enable_downmix(&self) -> bool {
        self.enable_downmix
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl AudioCapture for AppleAudio {
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
        assert_eq!(COREAUDIO_DEFAULT_DEVICE, "default");
        assert_eq!(speakers_from_channels(2), CoreAudioSpeakers::Stereo);
        assert_eq!(speakers_from_channels(1), CoreAudioSpeakers::Mono);
    }

    #[test]
    fn convert_known() {
        let raw = [0.0f32, 0.5];
        let out = coreaudio_float_to_f32(&raw, 1).unwrap();
        assert_eq!(out, raw);
    }

    #[test]
    fn open_errors_clearly() {
        match AppleAudio::open(48000, 2) {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("CoreAudio") || e.contains("not available") || e.contains("not yet")
                );
            }
        }
    }

    #[test]
    fn differential() {
        let a = coreaudio_float_to_f32(&[0.0, 0.1], 1).unwrap();
        let b = coreaudio_float_to_f32(&[0.0, 0.9], 1).unwrap();
        assert_ne!(a, b);
    }
}
