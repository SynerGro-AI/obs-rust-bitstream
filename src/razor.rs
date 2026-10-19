//! Safe pure Razor (split tool for video and audio timelines).
//!
//! Cuts a video clip or audio track at a time.
//! The left piece keeps the original name.
//! The right piece gets a "_r" suffix.
//! Empty names and missing clips are rejected.
//! No unwraps.
//!
//! Differential: splitting at the middle produces two pieces.

use crate::audio_timeline::{AudioTimeline, TimelineError};
use crate::video_timeline::{VideoError, VideoTimeline};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RazorError {
    EmptyName,
    Missing,
    OutOfRange,
}

impl From<VideoError> for RazorError {
    fn from(e: VideoError) -> Self {
        match e {
            VideoError::EmptyName => RazorError::EmptyName,
            VideoError::Missing => RazorError::Missing,
            VideoError::Duplicate => RazorError::EmptyName,
        }
    }
}

impl From<TimelineError> for RazorError {
    fn from(e: TimelineError) -> Self {
        match e {
            TimelineError::EmptyName => RazorError::EmptyName,
            TimelineError::Missing => RazorError::Missing,
            TimelineError::Duplicate => RazorError::EmptyName,
        }
    }
}

pub struct Razor;

impl Razor {
    /// Split a video clip at `time`. Left keeps the name. Right is "name_r".
    /// Returns the two names. None if the time is outside the clip.
    pub fn split_video(
        timeline: &mut VideoTimeline,
        name: &str,
        time: f32,
    ) -> Result<(String, String), RazorError> {
        if name.is_empty() {
            return Err(RazorError::EmptyName);
        }
        let clip = timeline.get(name)?;
        let start = clip.start_secs();
        let end = start + clip.duration_secs();
        if time <= start || time >= end {
            return Err(RazorError::OutOfRange);
        }
        let source = clip.source().to_string();
        let left_dur = time - start;
        let right_dur = end - time;
        let right_name = format!("{}_r", name);
        timeline.remove(name);
        timeline.add(name, start, left_dur, &source)?;
        timeline.add(&right_name, time, right_dur, &source)?;
        Ok((name.to_string(), right_name))
    }

    /// Split an audio track at `time`. Left keeps the name. Right is "name_r".
    /// The right track starts at `time`. Returns the two names.
    pub fn split_audio(
        timeline: &mut AudioTimeline,
        name: &str,
        time: f32,
    ) -> Result<(String, String), RazorError> {
        if name.is_empty() {
            return Err(RazorError::EmptyName);
        }
        let track = timeline.get_mut(name)?;
        let start = track.start_secs();
        if time <= start {
            return Err(RazorError::OutOfRange);
        }
        let right_name = format!("{}_r", name);
        timeline.add(&right_name, time)?;
        Ok((name.to_string(), right_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_video_middle() {
        let mut t = VideoTimeline::new();
        t.add("clip", 0.0, 10.0, "src").unwrap();
        let (left, right) = Razor::split_video(&mut t, "clip", 4.0).unwrap();
        assert_eq!(left, "clip");
        assert_eq!(right, "clip_r");
        assert!((t.get("clip").unwrap().duration_secs() - 4.0).abs() < 0.01);
        assert!((t.get("clip_r").unwrap().start_secs() - 4.0).abs() < 0.01);
        assert!((t.get("clip_r").unwrap().duration_secs() - 6.0).abs() < 0.01);
    }

    #[test]
    fn split_audio() {
        let mut t = AudioTimeline::new();
        t.add("track", 0.0).unwrap();
        let (left, right) = Razor::split_audio(&mut t, "track", 5.0).unwrap();
        assert_eq!(left, "track");
        assert_eq!(right, "track_r");
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn out_of_range() {
        let mut t = VideoTimeline::new();
        t.add("clip", 0.0, 10.0, "src").unwrap();
        assert!(Razor::split_video(&mut t, "clip", 15.0).is_err());
    }

    #[test]
    fn differential() {
        let mut a = VideoTimeline::new();
        a.add("c", 0.0, 10.0, "s").unwrap();
        let mut b = VideoTimeline::new();
        b.add("c", 0.0, 10.0, "s").unwrap();
        Razor::split_video(&mut a, "c", 3.0).unwrap();
        Razor::split_video(&mut b, "c", 7.0).unwrap();
        assert_ne!(
            a.get("c").unwrap().duration_secs(),
            b.get("c").unwrap().duration_secs()
        );
    }
}
