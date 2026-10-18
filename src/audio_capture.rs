//! Safe pure AudioCapture (audio frame source core).
//!
//! A trait for anything that produces audio samples.
//! SyntheticAudio generates a quiet tone. No unwraps.
//!
//! Differential: two calls differ.

#[derive(Clone, Debug)]
pub struct AudioSamples {
    pub rate: u32,
    pub channels: u16,
    pub data: Vec<f32>,
}

pub trait AudioCapture {
    fn samples(&mut self, frames: usize) -> Option<AudioSamples>;
    fn rate(&self) -> u32;
    fn channels(&self) -> u16;
}

/// Always-available synthetic audio source.
#[derive(Clone, Debug)]
pub struct SyntheticAudio {
    rate: u32,
    channels: u16,
    phase: f32,
}

impl SyntheticAudio {
    pub fn new(rate: u32, channels: u16) -> Self {
        Self {
            rate,
            channels,
            phase: 0.0,
        }
    }
}

impl AudioCapture for SyntheticAudio {
    fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
        if frames == 0 || self.rate == 0 || self.channels == 0 {
            return None;
        }
        let mut data = Vec::with_capacity(frames * self.channels as usize);
        for _ in 0..frames {
            let s = (self.phase * 0.1).sin() * 0.1;
            for _ in 0..self.channels {
                data.push(s);
            }
            self.phase += 1.0;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_produces() {
        let mut a = SyntheticAudio::new(48000, 2);
        let s = a.samples(16).unwrap();
        assert_eq!(s.data.len(), 32);
        assert_eq!(s.rate, 48000);
    }

    #[test]
    fn zero_frames() {
        let mut a = SyntheticAudio::new(48000, 1);
        assert!(a.samples(0).is_none());
    }

    #[test]
    fn differential() {
        let mut a = SyntheticAudio::new(48000, 1);
        let s1 = a.samples(4).unwrap();
        let s2 = a.samples(4).unwrap();
        assert_ne!(s1.data, s2.data);
    }
}
