//! Safe pure Scene (named source list).
//!
//! A scene owns a name and a SourceList. No unwraps.
//!
//! Differential: an added source can be found in the scene.

use crate::capture_select::CaptureSource;
use crate::source_list::SourceList;

pub struct Scene {
    name: String,
    sources: SourceList,
    audio: Option<String>,
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sources: SourceList::new(),
            audio: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sources(&self) -> &SourceList {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut SourceList {
        &mut self.sources
    }

    /// Remember which audio source this scene uses.
    pub fn set_audio(&mut self, name: &str) {
        self.audio = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
    }

    /// Audio source name this scene prefers, if any.
    pub fn audio(&self) -> Option<&str> {
        self.audio.as_deref()
    }

    /// Add a source to this scene.
    pub fn add(&mut self, name: &str, src: CaptureSource) -> usize {
        self.sources.add(name, src)
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// One line for the scene.
    pub fn summary(&self) -> String {
        format!(
            "{} | {} sources | active {} | audio {}",
            self.name,
            self.len(),
            self.sources.active_name().unwrap_or("none"),
            self.audio.as_deref().unwrap_or("-")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut s = Scene::new("main");
        assert_eq!(s.name(), "main");
        s.add("cam", CaptureSource::synthetic(16, 16));
        assert_eq!(s.len(), 1);
        assert!(s.sources().contains("cam"));
        assert!(s.summary().contains("main"));
        assert!(s.summary().contains("1 sources"));
        s.set_audio("mic");
        assert!(s.summary().contains("audio mic"));
    }
}
