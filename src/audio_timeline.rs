//! Safe pure AudioTimeline (multi-track core).
//!
//! Holds named tracks. Each track has a start time and a
//! channel. Empty and duplicate names are rejected. No unwraps.
//!
//! Differential: two tracks + active-at-time round-trips.

use std::collections::HashMap;

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
}
