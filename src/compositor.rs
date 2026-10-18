//! Safe pure Compositor (video frame core).
//!
//! Takes a video timeline and a time and produces the active
//! source for that frame. Can paint a real Y800 VideoFrame
//! onto a canvas as grayscale. No unwraps.
//!
//! Differential: two clips + compose at a time round-trips.
//! A known Y800 frame paints the expected luma.

use crate::capture::Capture;
use crate::pixel_canvas::PixelCanvas;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
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
    /// source paints a solid color so the frame is visible
    /// when no real pixels are available.
    pub fn paint(timeline: &VideoTimeline, time: f32, canvas: &mut PixelCanvas) {
        if let Some(src) = Self::compose(timeline, time) {
            let (r, g, b) = color_for(&src);
            canvas.fill(r, g, b);
        }
    }

    /// Paint a real Y800 frame onto the canvas as grayscale.
    /// Returns false if the frame is not Y800 or sizes do not match.
    pub fn paint_frame(frame: &VideoFrame, canvas: &mut PixelCanvas) -> bool {
        if frame.format != VideoFormat::Y800 {
            return false;
        }
        if frame.width != canvas.width() || frame.height != canvas.height() {
            return false;
        }
        let plane = match frame.planes.first() {
            Some(p) => p,
            None => return false,
        };
        let w = frame.width as usize;
        for y in 0..frame.height {
            for x in 0..frame.width {
                let i = (y as usize) * w + (x as usize);
                let y8 = plane.get(i).copied().unwrap_or(0);
                let v = y8 as f32 / 255.0;
                let _ = canvas.set(x, y, v, v, v);
            }
        }
        true
    }

    /// Pull one frame from a Capture and paint it.
    /// Returns false if the capture yields nothing or sizes mismatch.
    pub fn paint_from(capture: &mut dyn Capture, canvas: &mut PixelCanvas) -> bool {
        match capture.frame() {
            Some(frame) => Self::paint_frame(&frame, canvas),
            None => false,
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
    use crate::capture::SyntheticCapture;

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

    #[test]
    fn paint_frame_y800() {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        frame.planes[0] = vec![0, 128, 255, 64];
        let mut c = PixelCanvas::new(2, 2).unwrap();
        assert!(Compositor::paint_frame(&frame, &mut c));
        let (r, _, _) = c.get(1, 0).unwrap();
        assert!((r - 128.0 / 255.0).abs() < 0.01);
        let (r2, _, _) = c.get(0, 1).unwrap();
        assert!((r2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn paint_from_capture() {
        let mut cap = SyntheticCapture::new(4, 4);
        let mut c = PixelCanvas::new(4, 4).unwrap();
        assert!(Compositor::paint_from(&mut cap, &mut c));
        let (r, g, b) = c.get(0, 0).unwrap();
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn mismatch_safe() {
        let frame = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        assert!(!Compositor::paint_frame(&frame, &mut c));
    }
}