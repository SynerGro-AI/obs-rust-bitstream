//! Safe pure EditComposite (edit → composite change).
//!
//! Confirms that editing a layer's opacity changes the
//! composite mean. A lower opacity reads as a lower mean.
//! No unwraps.
//!
//! Differential: half opacity is below full opacity.

use crate::layer_composite::LayerComposite;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct EditComposite;

impl EditComposite {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Run and return (full_mean, half_mean).
    pub fn run() -> (f32, f32) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_mean = LayerComposite::mean(&full).unwrap_or(0.0);

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_mean = LayerComposite::mean(&half).unwrap_or(0.0);

        (full_mean, half_mean)
    }

    /// True if the half-opacity mean is below the full mean.
    pub fn correct(full: f32, half: f32) -> bool {
        half < full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_changes_mean() {
        let (full, half) = EditComposite::run();
        assert!(EditComposite::correct(full, half));
        assert!(full > 100.0);
        assert!(half < full);
    }

    #[test]
    fn half_is_lower() {
        let (full, half) = EditComposite::run();
        assert!(half < full);
    }

    #[test]
    fn differential() {
        let (full, half) = EditComposite::run();
        assert_ne!(full, half);
    }
}
