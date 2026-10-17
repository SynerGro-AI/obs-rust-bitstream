//! Safe pure AudioTimeline (multi-track core).
//!
//! Holds named tracks. Each track has a start time and a
//! channel. Empty and duplicate names are rejected. No unwraps.
//!
//! Differential: two tracks + active-at-time round-trips.

use std::collections::HashMap;

use crate::ai::AiAssistant;
use crate::wav_reader::WavData;
use crate::audio_bus::AudioBus;
use crate::audio_channel::{AudioChannel, ChannelError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimelineError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<ChannelError> for TimelineError {
    fn from(_: ChannelError) -> Self {
        TimelineError::EmptyName
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    name: String,
    start_secs: f32,
    channel: AudioChannel,
}

impl Track {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn start_secs(&self) -> f32 {
        self.start_secs
    }
    pub fn channel(&self) -> &AudioChannel {
        &self.channel
    }
    pub fn channel_mut(&mut self) -> &mut AudioChannel {
        &mut self.channel
    }
}

#[derive(Clone, Debug, Default)]
pub struct AudioTimeline {
    tracks: HashMap<String, Track>,
}

impl AudioTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, start_secs: f32) -> Result<(), TimelineError> {
        if name.is_empty() {
            return Err(TimelineError::EmptyName);
        }
        if self.tracks.contains_key(name) {
            return Err(TimelineError::Duplicate);
        }
        let channel = AudioChannel::new(name)?;
        self.tracks.insert(
            name.to_string(),
            Track {
                name: name.to_string(),
                start_secs,
                channel,
            },
        );
        Ok(())
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Track, TimelineError> {
        self.tracks.get_mut(name).ok_or(TimelineError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.tracks.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Names of tracks that have started by `time`.
    pub fn active_at(&self, time: f32) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .tracks
            .values()
            .filter(|t| t.start_secs <= time)
            .map(|t| t.name.as_str())
            .collect();
        names.sort();
        names
    }

    /// Ask the assistant for a track name and add it at a time.
    /// Falls back to `fallback` if the reply has no usable token.
    pub fn add_ai(
        &mut self,
        ai: &AiAssistant,
        prompt: &str,
        start_secs: f32,
        fallback: &str,
    ) -> Result<(), TimelineError> {
        let reply = ai.assist(prompt);
        let name = first_token(&reply).unwrap_or(fallback);
        if name.is_empty() {
            return Err(TimelineError::EmptyName);
        }
        self.add(name, start_secs)
    }

    /// Build a bus from every track that is active at `time`.
    pub fn build_bus(&self, time: f32) -> AudioBus {
        let mut bus = AudioBus::new();
        for name in self.active_at(time) {
            let _ = bus.add(name);
            if let (Some(track), Ok(ch)) = (self.tracks.get(name), bus.get_mut(name)) {
                ch.set_gain(track.channel.gain());
                ch.set_mute(track.channel.mute());
            }
        }
        bus
    }

    /// Render the mix level of the active tracks at `time`.
    pub fn render(&self, time: f32) -> f32 {
        let mut sum = 0.0;
        let mut has_samples = false;
        for name in self.active_at(time) {
            if let Some(track) = self.tracks.get(name) {
                let local = time - track.start_secs();
                let s = track.channel().sample_at(local);
                if s != 0.0 {
                    has_samples = true;
                }
                sum += s;
            }
        }
        if has_samples {
            sum
        } else {
            self.build_bus(time).mix_level()
        }
    }

    /// Load WAV samples into a track.
    pub fn load_wav(&mut self, name: &str, wav: &WavData) -> Result<(), TimelineError> {
        let track = self.get_mut(name)?;
        track.channel_mut().set_samples(wav.samples.clone(), wav.sample_rate);
        Ok(())
    }
}

fn first_token(s: &str) -> Option<&str> {
    s.split_whitespace()
        .find(|t| !t.starts_with('[') && !t.contains('=') && !t.contains('|'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut t = AudioTimeline::new();
        t.add("voice", 0.0).unwrap();
        t.add("music", 2.0).unwrap();
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut t = AudioTimeline::new();
        t.add("a", 0.0).unwrap();
        assert!(matches!(t.add("a", 0.0), Err(TimelineError::Duplicate)));
        assert!(matches!(t.add("", 0.0), Err(TimelineError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut t = AudioTimeline::new();
        t.add("voice", 0.0).unwrap();
        t.add("music", 2.0).unwrap();
        assert_eq!(t.active_at(1.0), vec!["voice"]);
        assert_eq!(t.active_at(3.0).len(), 2);
    }

    #[test]
    fn ai_edit_build_render() {
        let mut t = AudioTimeline::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        t.add_ai(&ai, "suggest a track", 0.0, "voice").unwrap();
        t.add("music", 1.0).unwrap();
        assert_eq!(t.len(), 2);
        let bus = t.build_bus(0.5);
        assert_eq!(bus.len(), 1);
        assert!(t.render(2.0) > 0.0);
    }
}
