//! Safe pure AudioSync (audio locked to video core).
//!
//! Pairs a TimedSequence with an AudioTimeline.
//! Sample returns the audio value at a frame's time.
//! No unwraps.
//!
//! Differential: frame 0 is time 0, later frames advance.

use crate::audio_timeline::AudioTimeline;
use crate::timed_sequence::TimedSequence;

#[derive(Clone, Debug)]
pub struct AudioSync {
    video: TimedSequence,
    audio: AudioTimeline,
}

impl AudioSync {
    pub fn new(video: TimedSequence) -> Self {
        Self {
            video,
            audio: AudioTimeline::new(),
        }
    }

    pub fn video(&self) -> &TimedSequence {
        &self.video
    }

    pub fn audio_mut(&mut self) -> &mut AudioTimeline {
        &mut self.audio
    }

    /// Time in seconds for a frame index.
    pub fn time_at(&self, frame: usize) -> f32 {
        if self.video.fps() <= 0.0 {
            return 0.0;
        }
        frame as f32 / self.video.fps()
    }

    /// Audio value at a frame. Missing audio is 0.
    pub fn sample(&self, frame: usize) -> f32 {
        let t = self.time_at(frame);
        self.audio.render(t)
    }

    /// Fill a slice of samples from the timeline.
    pub fn fill(&self, start_frame: usize, count: usize) -> Vec<f32> {
        (0..count).map(|i| self.sample(start_frame + i)).collect()
    }

    /// Frame index for a time in seconds.
    pub fn frame_at(&self, time: f32) -> usize {
        if self.video.fps() <= 0.0 {
            return 0;
        }
        (time * self.video.fps()).floor().max(0.0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    #[test]
    fn differential() {
        let mut video = TimedSequence::new(30.0).unwrap();
        for _ in 0..30 {
            let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
            video.push(f);
        }
        let sync = AudioSync::new(video);
        assert_eq!(sync.time_at(0), 0.0);
        assert!((sync.time_at(30) - 1.0).abs() < 0.01);
        assert_eq!(sync.frame_at(0.5), 15);
        assert_eq!(sync.sample(0), 0.0);
    }
}
