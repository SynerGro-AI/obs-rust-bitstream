//! Safe pure Session (combined video + audio core).
//!
//! Owns a video timeline and an audio timeline. emit produces
//! both the video frame and the audio mix at a time.
//! No unwraps.
//!
//! Differential: a clip and a track emit together.

use crate::audio_output::{AudioFrame, AudioOutput};
use crate::audio_timeline::AudioTimeline;
use crate::output::{Output, OutputFrame};
use crate::pixel_canvas::PixelCanvas;
use crate::video_timeline::VideoTimeline;

#[derive(Clone, Debug)]
pub struct SessionFrame {
    pub video: OutputFrame,
    pub audio: AudioFrame,
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    video: VideoTimeline,
    audio: AudioTimeline,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn video_mut(&mut self) -> &mut VideoTimeline {
        &mut self.video
    }

    pub fn audio_mut(&mut self) -> &mut AudioTimeline {
        &mut self.audio
    }

    /// Emit both the video frame and the audio mix at `time`.
    pub fn emit(&self, time: f32, canvas: &mut PixelCanvas) -> SessionFrame {
        SessionFrame {
            video: Output::emit_timeline(&self.video, time, canvas),
            audio: AudioOutput::emit(&self.audio, time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit() {
        let mut s = Session::new();
        s.video_mut().add("intro", 0.0, 2.0, "cam1").unwrap();
        s.audio_mut().add("voice", 0.0).unwrap();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        let f = s.emit(1.0, &mut c);
        assert_eq!(f.video.first, (1.0, 0.0, 0.0));
        assert_eq!(f.audio.tracks, 1);
    }

    #[test]
    fn differential() {
        let mut s = Session::new();
        s.video_mut().add("main", 0.0, 5.0, "cam2").unwrap();
        s.audio_mut().add("music", 0.0).unwrap();
        let mut c = PixelCanvas::new(2, 2).unwrap();
        let f = s.emit(1.0, &mut c);
        assert_eq!(f.video.first.1, 1.0);
        assert!(f.audio.level > 0.0);
    }
}
