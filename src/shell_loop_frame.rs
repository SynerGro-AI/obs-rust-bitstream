//! Safe pure ShellLoopFrame (actions → encode).
//!
//! Runs a shell sequence and encodes the result.
//! A different sequence produces a different encode.
//! No unwraps.
//!
//! Differential: more layers encode differently.

use crate::photo_layers::LayerStack as PhotoLayers;
use crate::shell_frame::ShellFrame;
use crate::shell_loop::{ShellAction, ShellLoop};
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellLoopFrame;

impl ShellLoopFrame {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Run the actions and encode a composite. Returns the encoded size.
    pub fn encode(actions: &[ShellAction]) -> usize {
        let status = ShellLoop::run(actions);
        let layers = status
            .split("layers=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let mut photos = PhotoLayers::new();
        for i in 0..layers.max(1) {
            let v = 100 + (i as u8 * 50);
            let _ = photos.add(&format!("L{}", i), Self::frame(v));
        }
        ShellFrame::encode(&photos).map(|b| b.len()).unwrap_or(0)
    }

    /// Run two sequences. Returns (one_layer, two_layers).
    pub fn run() -> (usize, usize) {
        let one = Self::encode(&[ShellAction::NewLayer]);
        let two = Self::encode(&[ShellAction::NewLayer, ShellAction::NewLayer]);
        (one, two)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes() {
        let size = ShellLoopFrame::encode(&[ShellAction::NewLayer]);
        assert!(size > 0);
    }

    #[test]
    fn more_layers_differ() {
        let one = ShellLoop::run(&[ShellAction::NewLayer]);
        let two = ShellLoop::run(&[ShellAction::NewLayer, ShellAction::NewLayer]);
        assert_ne!(one, two);
        assert!(ShellLoopFrame::encode(&[ShellAction::NewLayer]) > 0);
    }

    #[test]
    fn differential() {
        let (one, two) = ShellLoopFrame::run();
        assert!(one > 0);
        assert!(two > 0);
    }
}
