//! Safe pure Compositor (video frame core).
//!
//! Takes a video timeline and a time and produces the active
//! source for that frame. No unwraps.
//!
//! Differential: two clips + compose at a time round-trips.

use crate::pixel_canvas::PixelCanvas;
use crate::video_timeline::VideoTimeline;

#[derive(Clone, Debug)]
pub struct Compositor;

impl Compositor {
    /// Compose the frame at `time`. Returns the active source,
    /// or None if no clip covers the time.
    pub fn compose(timeline: &VideoTimeline, time: f32) -> Option<String> {
        timeline.build(time).into_iter().next()
    }

    /// Number of clips contributing to the frame.
    pub fn layer_count(timeline: &VideoTimeline, time: f32) -> usize {
        timeline.render(time)
    }

    /// Paint the composed frame onto a canvas. The active
    /// source paints a solid color so the frame is visible.
    pub fn paint(timeline: &VideoTimeline, time: f32, canvas: &mut PixelCanvas) {
        if let Some(src) = Self::compose(timeline, time) {
            let (r, g, b) = color_for(&src);
            canvas.fill(r, g, b);
        }
    }
}

fn color_for(source: &str) -> (f32, f32, f32) {
    match source {
        "cam1" => (1.0, 0.0, 0.0),
        "cam2" => (0.0, 1.0, 0.0),
        _ => (0.2, 0.2, 0.2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose() {
        let mut t = VideoTimeline::new();
        t.add("intro", 0.0, 2.0, "cam1").unwrap();
        t.add("main", 2.0, 5.0, "cam2").unwrap();
        assert_eq!(Compositor::compose(&t, 1.0), Some("cam1".to_string()));
        assert_eq!(Compositor::compose(&t, 3.0), Some("cam2".to_string()));
    }

    #[test]
    fn empty() {
        let t = VideoTimeline::new();
        assert!(Compositor::compose(&t, 0.0).is_none());
    }

    #[test]
    fn differential() {
        let mut t = VideoTimeline::new();
        t.add("a", 0.0, 1.0, "cam").unwrap();
        assert_eq!(Compositor::layer_count(&t, 0.5), 1);
        assert_eq!(Compositor::layer_count(&t, 2.0), 0);
    }

    #[test]
    fn paint() {
        let mut t = VideoTimeline::new();
        t.add("intro", 0.0, 2.0, "cam1").unwrap();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        Compositor::paint(&t, 1.0, &mut c);
        let (r, _, _) = c.get(0, 0).unwrap();
        assert_eq!(r, 1.0);
    }
}
