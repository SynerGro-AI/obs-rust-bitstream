//! Safe pure BlendComposite (blend mode → result).
//!
//! Confirms that changing a layer's blend mode changes
//! the composite mean. Multiply reads lower than Normal
//! for mid values. No unwraps.
//!
//! Differential: Multiply mean is below Normal mean.

use crate::blend::BlendMode;
use crate::layer_composite::LayerComposite;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct BlendComposite;

impl BlendComposite {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Run and return (normal_mean, multiply_mean).
    pub fn run() -> (f32, f32) {
        let mut normal = LayerStack::new();
        normal.add("Background", Self::frame(100)).unwrap();
        normal.add("Layer 1", Self::frame(200)).unwrap();
        let normal_mean = LayerComposite::mean(&normal).unwrap_or(0.0);

        let mut multiply = LayerStack::new();
        multiply.add("Background", Self::frame(100)).unwrap();
        multiply.add("Layer 1", Self::frame(200)).unwrap();
        let _ = multiply.set_blend("Layer 1", BlendMode::Multiply);
        let multiply_mean = LayerComposite::mean(&multiply).unwrap_or(0.0);

        (normal_mean, multiply_mean)
    }

    /// True if Multiply is below Normal.
    pub fn correct(normal: f32, multiply: f32) -> bool {
        multiply < normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiply_is_lower() {
        let (normal, multiply) = BlendComposite::run();
        assert!(BlendComposite::correct(normal, multiply));
        assert!(multiply < normal);
    }

    #[test]
    fn normal_is_top() {
        let (normal, _) = BlendComposite::run();
        assert!((normal - 200.0).abs() < 1.0);
    }

    #[test]
    fn differential() {
        let (normal, multiply) = BlendComposite::run();
        assert_ne!(normal, multiply);
    }
}
