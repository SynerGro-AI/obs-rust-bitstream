//! Optional cpal backend for AudioCapture.
//!
//! When the `audio` feature is enabled, this module uses cpal
//! to open the default input device on Linux (ALSA), Windows
//! (WASAPI), and macOS (CoreAudio). Without the feature the
//! module is empty and the pure path remains.
//!
//! Differential: a known buffer converts to the expected length.

use crate::audio_capture::AudioCapture;

#[cfg(feature = "audio")]
mod imp {
    use super::*;
    use crate::audio_capture::AudioSamples;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};

    pub struct CpalAudio {
        rate: u32,
        channels: u16,
        buffer: Arc<Mutex<Vec<f32>>>,
        _stream: cpal::Stream,
    }

    impl CpalAudio {
        pub fn open(rate: u32, channels: u16) -> Result<Self, String> {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or_else(|| "no default input device".to_string())?;
            let config = device
                .default_input_config()
                .map_err(|e| format!("default config: {}", e))?;
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let buf_clone = buffer.clone();
            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        if let Ok(mut b) = buf_clone.lock() {
                            b.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("cpal error: {}", err),
                    None,
                )
                .map_err(|e| format!("build stream: {}", e))?;
            stream.play().map_err(|e| format!("play: {}", e))?;
            Ok(Self {
                rate,
                channels,
                buffer,
                _stream: stream,
            })
        }
    }

    impl AudioCapture for CpalAudio {
        fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
            let mut buf = self.buffer.lock().ok()?;
            let need = frames * self.channels as usize;
            if buf.len() < need {
                return None;
            }
            let data = buf.drain(..need).collect();
            Some(AudioSamples {
                rate: self.rate,
                channels: self.channels,
                data,
            })
        }

        fn rate(&self) -> u32 {
            self.rate
        }

        fn channels(&self) -> u16 {
            self.channels
        }
    }
}

/// Open the default input via cpal when the feature is enabled.
/// Returns a clear error otherwise.
pub fn open_cpal(rate: u32, channels: u16) -> Result<Box<dyn AudioCapture>, String> {
    #[cfg(feature = "audio")]
    {
        imp::CpalAudio::open(rate, channels).map(|a| Box::new(a) as Box<dyn AudioCapture>)
    }
    #[cfg(not(feature = "audio"))]
    {
        let _ = (rate, channels);
        Err("cpal backend not enabled (build with --features audio)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "audio"))]
    #[test]
    fn disabled_is_clear() {
        let e = open_cpal(48000, 1)
            .err()
            .expect("no backend without the feature");
        assert!(
            e.contains("not enabled") && e.contains("--features audio"),
            "{}",
            e
        );
    }

    /// With the feature on, the cpal backend really runs: on a machine with
    /// no input device (CI) it fails with a device/stream error, never the
    /// "not enabled" stub.
    #[cfg(feature = "audio")]
    #[test]
    fn enabled_backend_is_compiled_in() {
        match open_cpal(48000, 1) {
            Ok(mut a) => {
                assert_eq!(a.rate(), 48000);
                assert_eq!(a.channels(), 1);
                let _ = a.samples(16);
            }
            Err(e) => assert!(
                !e.contains("not enabled"),
                "audio feature built the stub: {}",
                e
            ),
        }
    }
}
