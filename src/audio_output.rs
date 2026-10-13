//! Safe pure AudioOutput (mix emit core).
//!
//! Takes an audio timeline and a time and emits the mix level
//! of the active tracks. No unwraps.
//!
//! Differential: two tracks + emit at a time round-trips.

use crate::audio_timeline::AudioTimeline;

#[derive(Clone, Debug)]
pub struct AudioFrame {
    pub level: f32,
    pub tracks: usize,
}

pub struct AudioOutput;

impl AudioOutput {
    /// Emit the mix of tracks active at `time`.
    pub fn emit(timeline: &AudioTimeline, time: f32) -> AudioFrame {
        AudioFrame {
            level: timeline.render(time),
            tracks: timeline.active_at(time).len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit() {
        let mut t = AudioTimeline::new();
        t.add("voice", 0.0).unwrap();
        t.add("music", 2.0).unwrap();
        let f = AudioOutput::emit(&t, 1.0);
        assert_eq!(f.tracks, 1);
        assert!(f.level > 0.0);
    }

    #[test]
    fn differential() {
        let mut t = AudioTimeline::new();
        t.add("voice", 0.0).unwrap();
        t.add("music", 0.0).unwrap();
        let f = AudioOutput::emit(&t, 1.0);
        assert_eq!(f.tracks, 2);
        assert!((f.level - 2.0).abs() < 0.001);
    }
}
