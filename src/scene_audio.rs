//! Safe pure SceneAudio (bind scene audio to the source list).
//!
//! Applies a scene's remembered audio source name to an
//! AudioSourceList. Missing names are a no-op. No unwraps.
//!
//! Differential: switch + apply changes the active audio.

use crate::audio_source_list::AudioSourceList;
use crate::scene::Scene;

pub struct SceneAudio;

impl SceneAudio {
    /// Switch the audio list to the source the scene remembers.
    /// Returns true if the name existed and was switched.
    pub fn apply(scene: &Scene, audio: &mut AudioSourceList) -> bool {
        match scene.audio() {
            Some(name) => audio.switch(name),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;

    #[test]
    fn apply_switches() {
        let mut scene = Scene::new("main");
        scene.set_audio("mic");
        let mut audio = AudioSourceList::new();
        audio.add("mic", AudioSource::synthetic(48000, 1));
        audio.add("line", AudioSource::synthetic(48000, 1));
        audio.switch("line");
        assert!(SceneAudio::apply(&scene, &mut audio));
        assert_eq!(audio.active_name(), Some("mic"));
    }

    #[test]
    fn missing_is_noop() {
        let mut scene = Scene::new("main");
        scene.set_audio("ghost");
        let mut audio = AudioSourceList::new();
        audio.add("mic", AudioSource::synthetic(48000, 1));
        assert!(!SceneAudio::apply(&scene, &mut audio));
        assert_eq!(audio.active_name(), Some("mic"));
    }

    #[test]
    fn differential() {
        let mut a = Scene::new("a");
        a.set_audio("mic");
        let mut b = Scene::new("b");
        b.set_audio("line");
        let mut audio = AudioSourceList::new();
        audio.add("mic", AudioSource::synthetic(48000, 1));
        audio.add("line", AudioSource::synthetic(48000, 1));
        SceneAudio::apply(&a, &mut audio);
        let first = audio.active_name().unwrap().to_string();
        SceneAudio::apply(&b, &mut audio);
        assert_ne!(first, audio.active_name().unwrap());
    }
}
