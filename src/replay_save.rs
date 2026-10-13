//! Safe pure ReplaySave (replay to clip core).
//!
//! Saves the replay buffer to a named clip. Empty names
//! rejected. No unwraps.
//!
//! Differential: save + read round-trips.

use crate::instant_replay::InstantReplay;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    EmptyName,
    EmptyBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFormat {
    Clip,
    Gif,
    Video,
}

#[derive(Clone, Debug)]
pub struct SavedClip {
    pub frames: Vec<u64>,
    pub format: SaveFormat,
}

#[derive(Clone, Debug, Default)]
pub struct ReplaySave {
    clips: BTreeMap<String, SavedClip>,
}

impl ReplaySave {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save the current replay buffer as a named clip in a format.
    pub fn save_as(
        &mut self,
        name: &str,
        replay: &InstantReplay,
        format: SaveFormat,
    ) -> Result<(), SaveError> {
        if name.is_empty() {
            return Err(SaveError::EmptyName);
        }
        if replay.len() == 0 {
            return Err(SaveError::EmptyBuffer);
        }
        self.clips.insert(
            name.to_string(),
            SavedClip {
                frames: replay.frames().to_vec(),
                format,
            },
        );
        Ok(())
    }

    /// Save as a clip.
    pub fn save(&mut self, name: &str, replay: &InstantReplay) -> Result<(), SaveError> {
        self.save_as(name, replay, SaveFormat::Clip)
    }

    pub fn saved(&self, name: &str) -> Option<&SavedClip> {
        self.clips.get(name)
    }

    pub fn clip(&self, name: &str) -> Option<&[u64]> {
        self.clips.get(name).map(|c| c.frames.as_slice())
    }

    pub fn count(&self) -> usize {
        self.clips.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = ReplaySave::new();
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut s = ReplaySave::new();
        let r = InstantReplay::new(3);
        assert!(s.save("", &r).is_err());
    }

    #[test]
    fn differential() {
        let mut s = ReplaySave::new();
        let mut r = InstantReplay::new(3);
        r.push(1);
        r.push(2);
        s.save("goal", &r).unwrap();
        assert_eq!(s.clip("goal"), Some(&[1, 2][..]));
    }

    #[test]
    fn formats() {
        let mut s = ReplaySave::new();
        let mut r = InstantReplay::new(3);
        r.push(1);
        s.save_as("loop", &r, SaveFormat::Gif).unwrap();
        s.save_as("highlight", &r, SaveFormat::Video).unwrap();
        assert_eq!(s.saved("loop").unwrap().format, SaveFormat::Gif);
        assert_eq!(s.saved("highlight").unwrap().format, SaveFormat::Video);
    }
}
