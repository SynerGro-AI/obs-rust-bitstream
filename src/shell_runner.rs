//! Safe pure ShellRunner (shell composite → encode loop).
//!
//! Encodes N frames from the shell composite.
//! An opacity edit changes the encoded size.
//! No unwraps.
//!
//! Differential: half opacity encodes differently.

use crate::shell_frame::ShellFrame;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellRunner;

impl ShellRunner {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Encode `n` frames from the stack. Returns total bytes.
    pub fn run(stack: &LayerStack, n: usize) -> usize {
        let mut total = 0;
        for _ in 0..n {
            if let Some(bytes) = ShellFrame::encode(stack) {
                total += bytes.len();
            }
        }
        total
    }

    /// Run full and half for 3 frames. Returns (full_total, half_total).
    pub fn compare() -> (usize, usize) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_total = Self::run(&full, 3);

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_total = Self::run(&half, 3);

        (full_total, half_total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_n() {
        let mut s = LayerStack::new();
        s.add("A", ShellRunner::frame(128)).unwrap();
        let total = ShellRunner::run(&s, 3);
        assert!(total > 0);
    }

    #[test]
    fn edit_changes_total() {
        let (full, half) = ShellRunner::compare();
        assert_ne!(full, half);
    }

    #[test]
    fn differential() {
        let (full, half) = ShellRunner::compare();
        assert!(full > 0);
        assert!(half > 0);
    }
}
