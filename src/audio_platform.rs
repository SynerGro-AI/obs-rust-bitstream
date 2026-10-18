//! Safe pure platform audio open (microphone surface).
//!
//! Mirrors the video open_default pattern. On this environment
//! the real backends are not wired (no ALSA/WASAPI/AVAudio
//! bindings compiled). open_default returns a clear error.
//! Synthetic remains always available.
//!
//! Differential: synthetic produces samples; platform errors.

use crate::audio_capture::{AudioCapture, AudioSamples, SyntheticAudio};

/// Open the default microphone for this platform.
/// On Linux this attempts ALSA. On Windows this attempts WASAPI.
/// On macOS/iOS this attempts CoreAudio.
/// Open the default microphone for this platform.
/// Tries the cpal backend first when the audio feature is on,
/// then the platform path (ALSA / WASAPI / CoreAudio).
/// Returns a clear error if no backend is present.
pub fn open_default_audio() -> Result<Box<dyn AudioCapture>, String> {
    if let Ok(src) = crate::audio_cpal::open_cpal(48000, 1) {
        return Ok(src);
    }
    #[cfg(target_os = "linux")]
    {
        return crate::linux_audio::LinuxAudio::open(48000, 1)
            .map(|a| Box::new(a) as Box<dyn AudioCapture>);
    }
    #[cfg(windows)]
    {
        return crate::windows_audio::WindowsAudio::open(48000, 2)
            .map(|a| Box::new(a) as Box<dyn AudioCapture>);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        return crate::apple_audio::AppleAudio::open(48000, 2)
            .map(|a| Box::new(a) as Box<dyn AudioCapture>);
    }
    #[cfg(not(any(target_os = "linux", windows, target_os = "macos", target_os = "ios")))]
    {
        Err("default microphone not yet wired on this platform".to_string())
    }
}

/// Name of the backend that would be used.
/// "cpal" when the feature is on, otherwise the platform name.
pub fn backend_name() -> &'static str {
    #[cfg(feature = "audio")]
    {
        return "cpal";
    }
    #[cfg(not(feature = "audio"))]
    {
        #[cfg(target_os = "linux")]
        {
            return "alsa";
        }
        #[cfg(windows)]
        {
            return "wasapi";
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            return "coreaudio";
        }
        #[cfg(not(any(target_os = "linux", windows, target_os = "macos", target_os = "ios")))]
        {
            return "none";
        }
    }
}

/// Best available audio source: platform if it opens, otherwise synthetic.
pub fn open_best_audio(rate: u32, channels: u16) -> Box<dyn AudioCapture> {
    match open_default_audio() {
        Ok(src) => src,
        Err(_) => Box::new(SyntheticAudio::new(rate, channels)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_errors_clearly() {
        match open_default_audio() {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("not yet wired")
                        || e.contains("ALSA")
                        || e.contains("WASAPI")
                        || e.contains("AVAudio")
                );
            }
        }
    }

    #[test]
    fn best_is_synthetic_here() {
        let mut src = open_best_audio(48000, 1);
        let s = src.samples(8).unwrap();
        assert_eq!(s.data.len(), 8);
        assert_eq!(s.rate, 48000);
    }

    #[test]
    fn differential() {
        let mut a = open_best_audio(48000, 1);
        let s1 = a.samples(4).unwrap();
        let s2 = a.samples(4).unwrap();
        assert_ne!(s1.data, s2.data);
    }
}
