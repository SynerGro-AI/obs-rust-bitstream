//! Safe pure AudioSourceList (ordered audio sources).
//!
//! Holds named audio sources and returns samples from the
//! active one. Synthetic is always available. No unwraps.
//!
//! Differential: an added source can be found by name.

use crate::audio_capture::{AudioCapture, AudioSamples, SyntheticAudio};

pub enum AudioSource {
    Synthetic(SyntheticAudio),
    /// Platform microphone (future). Holds any AudioCapture.
    Platform(Box<dyn AudioCapture>),
}

impl AudioSource {
    pub fn synthetic(rate: u32, channels: u16) -> Self {
        AudioSource::Synthetic(SyntheticAudio::new(rate, channels))
    }

    pub fn platform(src: Box<dyn AudioCapture>) -> Self {
        AudioSource::Platform(src)
    }

    pub fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
        match self {
            AudioSource::Synthetic(a) => a.samples(frames),
            AudioSource::Platform(a) => a.samples(frames),
        }
    }

    pub fn rate(&self) -> u32 {
        match self {
            AudioSource::Synthetic(a) => a.rate(),
            AudioSource::Platform(a) => a.rate(),
        }
    }

    pub fn channels(&self) -> u16 {
        match self {
            AudioSource::Synthetic(a) => a.channels(),
            AudioSource::Platform(a) => a.channels(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            AudioSource::Synthetic(_) => "synthetic",
            AudioSource::Platform(_) => "platform",
        }
    }
}

pub struct AudioSourceList {
    items: Vec<(String, AudioSource)>,
    active: usize,
}

impl AudioSourceList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
        }
    }

    /// Add a source. Returns its index.
    pub fn add(&mut self, name: &str, src: AudioSource) -> usize {
        self.items.push((name.to_string(), src));
        self.items.len() - 1
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.items.iter().any(|(n, _)| n == name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.items.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Set the active index. Clamped.
    pub fn set_active(&mut self, i: usize) {
        if !self.items.is_empty() {
            self.active = i.min(self.items.len() - 1);
        }
    }

    pub fn active_name(&self) -> Option<&str> {
        self.items.get(self.active).map(|(n, _)| n.as_str())
    }

    /// Switch to the source named. Returns true if found.
    pub fn switch(&mut self, name: &str) -> bool {
        if let Some(i) = self.items.iter().position(|(n, _)| n == name) {
            self.active = i;
            true
        } else {
            false
        }
    }

    /// Samples from the active source.
    pub fn samples(&mut self, frames: usize) -> Option<AudioSamples> {
        self.items.get_mut(self.active).and_then(|(_, s)| s.samples(frames))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_switch() {
        let mut list = AudioSourceList::new();
        list.add("mic", AudioSource::synthetic(48000, 1));
        list.add("tone", AudioSource::synthetic(48000, 2));
        assert!(list.switch("tone"));
        assert_eq!(list.active_name(), Some("tone"));
        let s = list.samples(8).unwrap();
        assert_eq!(s.channels, 2);
    }

    #[test]
    fn missing_switch() {
        let mut list = AudioSourceList::new();
        assert!(!list.switch("none"));
    }

    #[test]
    fn differential() {
        let mut list = AudioSourceList::new();
        list.add("mic", AudioSource::synthetic(48000, 1));
        let a = list.samples(4).unwrap();
        let b = list.samples(4).unwrap();
        assert_ne!(a.data, b.data);
    }
}
