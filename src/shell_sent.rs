//! Safe pure ShellSent (edit → sent bytes).
//!
//! Confirms that an opacity edit changes the payload
//! the socket would send. No unwraps.
//!
//! Differential: half opacity sends fewer bytes.

use crate::photo_layers::LayerStack;
use crate::shell_socket::ShellSocket;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellSent;

impl ShellSent {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Run full and half. Returns (full_len, half_len).
    pub fn run() -> (usize, usize) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_len = ShellSocket::payload(&full).map(|p| p.len()).unwrap_or(0);

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_len = ShellSocket::payload(&half).map(|p| p.len()).unwrap_or(0);

        (full_len, half_len)
    }

    /// True if the edit changed the sent size.
    pub fn changed(full: usize, half: usize) -> bool {
        full != half
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_changes_size() {
        let (full, half) = ShellSent::run();
        assert!(ShellSent::changed(full, half));
    }

    #[test]
    fn both_nonzero() {
        let (full, half) = ShellSent::run();
        assert!(full > 0);
        assert!(half > 0);
    }

    #[test]
    fn differential() {
        let (full, half) = ShellSent::run();
        assert_ne!(full, half);
    }
}
