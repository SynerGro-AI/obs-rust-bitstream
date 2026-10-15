//! Safe pure SceneRenderer (timeline → frames).
//!
//! Uses the existing VideoTimeline and Compositor to choose
//! a source at each time, then renders a solid frame for it.
//! No unwraps.
//!
//! Differential: two clips produce two different luminances.

use crate::color_source::ColorSource;
use crate::compositor::Compositor;
use crate::frame_renderer::{FrameRenderer, RenderError};
use crate::frame_sequence::FrameSequence;
use crate::video_frame::VideoFrame;
use crate::video_timeline::VideoTimeline;

pub struct SceneRenderer;

impl SceneRenderer {
    /// Color for a source name. Matches the compositor's
    /// known sources so the frame matches the scene.
    fn color_for(source: &str) -> ColorSource {
        match source {
            "cam1" => ColorSource::new(255, 0, 0),
            "cam2" => ColorSource::new(0, 255, 0),
            _ => ColorSource::new(32, 32, 32),
        }
    }

    /// Render one frame at `time`. Returns the frame and the
    /// source that was active.
    pub fn frame(
        timeline: &VideoTimeline,
        time: f32,
        width: u32,
        height: u32,
    ) -> Result<(VideoFrame, String), RenderError> {
        let source = Compositor::compose(timeline, time).unwrap_or_else(|| "none".to_string());
        let color = Self::color_for(&source);
        let frame = FrameRenderer::solid(&color, width, height)?;
        Ok((frame, source))
    }

    /// Render `count` frames starting at time 0, stepping by
    /// `dt` seconds. Pushes them into the sequence.
    pub fn sequence(
        timeline: &VideoTimeline,
        count: usize,
        dt: f32,
        width: u32,
        height: u32,
    ) -> Result<FrameSequence, RenderError> {
        let mut seq = FrameSequence::new();
        for i in 0..count {
            let t = i as f32 * dt;
            let (frame, _) = Self::frame(timeline, t, width, height)?;
            seq.push(frame);
        }
        Ok(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut t = VideoTimeline::new();
        t.add("intro", 0.0, 1.0, "cam1").unwrap();
        t.add("main", 1.0, 1.0, "cam2").unwrap();
        let (f1, s1) = SceneRenderer::frame(&t, 0.5, 2, 2).unwrap();
        let (f2, s2) = SceneRenderer::frame(&t, 1.5, 2, 2).unwrap();
        assert_eq!(s1, "cam1");
        assert_eq!(s2, "cam2");
        assert_ne!(f1.planes[0][0], f2.planes[0][0]);
    }
}
