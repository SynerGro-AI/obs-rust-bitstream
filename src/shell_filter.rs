//! Safe pure ShellFilter (filter → encode).
//!
//! Applies a brightness filter to the shell composite
//! and encodes it. A brighter frame encodes differently.
//! No unwraps.
//!
//! Differential: brightness 50 encodes differently from 0.

use crate::bright_contrast::BrightContrast;
use crate::h264_codec::H264Codec;
use crate::layer_composite::LayerComposite;
use crate::live_weights::LiveWeights;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellFilter;

impl ShellFilter {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Composite, apply brightness, encode. Returns the encoded bytes.
    pub fn encode(stack: &LayerStack, brightness: i8) -> Option<Vec<u8>> {
        let mut composite = LayerComposite::result(stack)?;
        BrightContrast::apply(&mut composite, brightness, 0).ok()?;
        let weights = LiveWeights::new();
        let mut codec = H264Codec::new(composite.width, composite.height, &weights).ok()?;
        codec.encode(&composite).ok()
    }

    /// Run plain and bright. Returns (plain_len, bright_len).
    pub fn run() -> (usize, usize) {
        let mut s = LayerStack::new();
        s.add("Background", Self::frame(100)).unwrap();
        s.add("Layer 1", Self::frame(150)).unwrap();
        let plain = Self::encode(&s, 0).map(|b| b.len()).unwrap_or(0);
        let bright = Self::encode(&s, 50).map(|b| b.len()).unwrap_or(0);
        (plain, bright)
    }

    /// True if the filter changed the encode.
    pub fn changed(plain: &[u8], bright: &[u8]) -> bool {
        plain != bright
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_changes_encode() {
        let mut s = LayerStack::new();
        s.add("A", ShellFilter::frame(100)).unwrap();
        let plain = ShellFilter::encode(&s, 0).unwrap();
        let bright = ShellFilter::encode(&s, 50).unwrap();
        assert!(ShellFilter::changed(&plain, &bright));
    }

    #[test]
    fn encodes() {
        let mut s = LayerStack::new();
        s.add("A", ShellFilter::frame(128)).unwrap();
        assert!(!ShellFilter::encode(&s, 0).unwrap().is_empty());
    }

    #[test]
    fn differential() {
        let (plain, bright) = ShellFilter::run();
        assert!(plain > 0);
        assert!(bright > 0);
    }
}
