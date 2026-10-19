//! Safe pure LayerComposite (visible layers → result).
//!
//! Composites the visible layers bottom to top using
//! each layer's blend mode. Hidden layers are skipped.
//! The eye controls what you see. No unwraps.
//!
//! Differential: hiding a layer changes the result.

use crate::blend::Blend;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct LayerComposite;

impl LayerComposite {
    /// Composite the visible layers. None if none are visible
    /// or the frames cannot blend.
    pub fn result(stack: &LayerStack) -> Option<VideoFrame> {
        let visible = stack.visible_layers();
        if visible.is_empty() {
            return None;
        }
        let first = visible[0];
        let mut base = first.frame.clone();
        // Apply opacity of the base by scaling toward black.
        Self::apply_opacity(&mut base, first.opacity);
        for over in visible.iter().skip(1) {
            let mut src = over.frame.clone();
            Self::apply_opacity(&mut src, over.opacity);
            Blend::apply(&mut base, &src, over.blend).ok()?;
        }
        Some(base)
    }

    fn apply_opacity(frame: &mut VideoFrame, opacity: u8) {
        if opacity == 255 {
            return;
        }
        let scale = opacity as f32 / 255.0;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = (*p as f32 * scale) as u8;
            }
        }
    }

    /// Mean of the composite. None if no visible layers.
    pub fn mean(stack: &LayerStack) -> Option<f32> {
        let frame = Self::result(stack)?;
        let plane = frame.planes.first()?;
        if plane.is_empty() {
            return None;
        }
        let sum: u32 = plane.iter().map(|p| *p as u32).sum();
        Some(sum as f32 / plane.len() as f32)
    }

    /// One line: "visible=N mean=N"
    pub fn line(stack: &LayerStack) -> String {
        let n = stack.visible_layers().len();
        match Self::mean(stack) {
            Some(m) => format!("visible={} mean={:.1}", n, m),
            None => format!("visible={} mean=none", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    #[test]
    fn composites_visible() {
        let mut s = LayerStack::new();
        s.add("Background", frame(100)).unwrap();
        s.add("Layer 1", frame(200)).unwrap();
        let mean = LayerComposite::mean(&s).unwrap();
        assert!(mean > 100.0);
        assert!(LayerComposite::line(&s).contains("visible=2"));
    }

    #[test]
    fn hide_changes_result() {
        let mut s = LayerStack::new();
        s.add("Background", frame(50)).unwrap();
        s.add("Layer 1", frame(200)).unwrap();
        let before = LayerComposite::mean(&s).unwrap();
        s.hide("Layer 1").unwrap();
        let after = LayerComposite::mean(&s).unwrap();
        assert!(after < before);
        assert!(LayerComposite::line(&s).contains("visible=1"));
    }

    #[test]
    fn all_hidden_is_none() {
        let mut s = LayerStack::new();
        s.add("A", frame(100)).unwrap();
        s.hide("A").unwrap();
        assert!(LayerComposite::result(&s).is_none());
        assert!(LayerComposite::line(&s).contains("mean=none"));
    }

    #[test]
    fn differential() {
        let mut a = LayerStack::new();
        let mut b = LayerStack::new();
        a.add("A", frame(50)).unwrap();
        b.add("A", frame(200)).unwrap();
        assert!(LayerComposite::mean(&b).unwrap() > LayerComposite::mean(&a).unwrap());
    }
}
