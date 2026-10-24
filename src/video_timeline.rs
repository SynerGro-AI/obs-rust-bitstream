//! Safe pure VideoTimeline (multi-clip video edit core).
//!
//! Holds named clips. Each clip has a start time, a duration,
//! and a source name. build produces the
//! active clip list. render returns the active clip count.
//! Empty and duplicate names rejected. No unwraps.
//!
//! Differential: two clips + active-at-time round-trips.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VideoError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Clip {
    name: String,
    start_secs: f32,
    duration_secs: f32,
    source: String,
}

impl Clip {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn start_secs(&self) -> f32 {
        self.start_secs
    }
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, Default)]
pub struct VideoTimeline {
    clips: HashMap<String, Clip>,
}

impl VideoTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        name: &str,
        start_secs: f32,
        duration_secs: f32,
        source: &str,
    ) -> Result<(), VideoError> {
        if name.is_empty() || source.is_empty() {
            return Err(VideoError::EmptyName);
        }
        if self.clips.contains_key(name) {
            return Err(VideoError::Duplicate);
        }
        self.clips.insert(
            name.to_string(),
            Clip {
                name: name.to_string(),
                start_secs,
                duration_secs: duration_secs.max(0.0),
                source: source.to_string(),
            },
        );
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Clip, VideoError> {
        self.clips.get(name).ok_or(VideoError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.clips.remove(name).is_some()
    }

    pub fn len(&self) -> usize {
        self.clips.len()
    }

    /// Clips that cover `time`.
    pub fn active_at(&self, time: f32) -> Vec<&Clip> {
        let mut clips: Vec<&Clip> = self
            .clips
            .values()
            .filter(|c| time >= c.start_secs && time < c.start_secs + c.duration_secs)
            .collect();
        clips.sort_by(|a, b| a.name.cmp(&b.name));
        clips
    }

    /// Build the list of active clip sources at a time.
    pub fn build(&self, time: f32) -> Vec<String> {
        self.active_at(time)
            .into_iter()
            .map(|c| c.source.clone())
            .collect()
    }

    /// Render: number of active clips at a time.
    pub fn render(&self, time: f32) -> usize {
        self.active_at(time).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut t = VideoTimeline::new();
        t.add("intro", 0.0, 2.0, "cam1").unwrap();
        t.add("main", 2.0, 5.0, "cam2").unwrap();
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn empty_name() {
        let mut t = VideoTimeline::new();
        assert!(t.add("", 0.0, 1.0, "cam").is_err());
    }

    #[test]
    fn differential() {
        let mut t = VideoTimeline::new();
        t.add("intro", 0.0, 2.0, "cam1").unwrap();
        t.add("main", 2.0, 5.0, "cam2").unwrap();
        assert_eq!(t.render(1.0), 1);
        assert_eq!(t.build(3.0), vec!["cam2".to_string()]);
    }
}
