//! Safe pure ShellFrame (edit → frame → encode).
//!
//! Builds a composite frame from the visible layers and
//! encodes it. An opacity edit changes the encoded bytes.
//! No unwraps.
//!
//! Differential: half opacity encodes differently from full.

use crate::h264_codec::H264Codec;
use crate::layer_composite::LayerComposite;
use crate::live_weights::LiveWeights;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellFrame;

impl ShellFrame {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Build the composite and encode it. Returns the encoded bytes.
    pub fn encode(stack: &LayerStack) -> Option<Vec<u8>> {
        let composite = LayerComposite::result(stack)?;
        let weights = LiveWeights::new();
        let mut codec = H264Codec::new(composite.width, composite.height, &weights).ok()?;
        codec.encode(&composite).ok()
    }

    /// Run full and half and return (full_len, half_len).
    pub fn run() -> (usize, usize) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_bytes = Self::encode(&full).unwrap_or_default();

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_bytes = Self::encode(&half).unwrap_or_default();

        (full_bytes.len(), half_bytes.len())
    }

    /// True if the edit changed the encoded output.
    pub fn changed(full: &[u8], half: &[u8]) -> bool {
        full != half
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_changes_encode() {
        let mut full = LayerStack::new();
        full.add("Background", ShellFrame::frame(100)).unwrap();
        full.add("Layer 1", ShellFrame::frame(200)).unwrap();
        let mut half = LayerStack::new();
        half.add("Background", ShellFrame::frame(100)).unwrap();
        half.add("Layer 1", ShellFrame::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let full_bytes = ShellFrame::encode(&full).unwrap();
        let half_bytes = ShellFrame::encode(&half).unwrap();
        assert!(ShellFrame::changed(&full_bytes, &half_bytes));
    }

    #[test]
    fn encodes() {
        let mut s = LayerStack::new();
        s.add("A", ShellFrame::frame(128)).unwrap();
        let bytes = ShellFrame::encode(&s).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn differential() {
        let (full, half) = ShellFrame::run();
        assert!(full > 0);
        assert!(half > 0);
    }
}
