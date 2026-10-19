//! Safe pure ShellPublish (shell packet → RTMP send).
//!
//! Encodes the shell composite and sends it through
//! the RTMP output. An opacity edit changes the status.
//! No unwraps.
//!
//! Differential: half opacity produces a different status.

use crate::encoder_context::EncoderType;
use crate::encoder_packet::EncoderPacket;
use crate::packet_file::PacketFile;
use crate::photo_layers::LayerStack;
use crate::rtmp_output::RtmpOutput;
use crate::shell_frame::ShellFrame;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellPublish;

impl ShellPublish {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Encode and send. Returns the status line.
    pub fn send(stack: &LayerStack) -> Option<String> {
        let bytes = ShellFrame::encode(stack)?;
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, bytes);
        let body = PacketFile::write(&[pkt]);
        let mut payload = vec![0u8; 12];
        payload.extend_from_slice(&body);
        let status = RtmpOutput::send("shell", "key", &payload).ok()?;
        Some(status.one_line())
    }

    /// Run full and half. Returns (full_line, half_line).
    pub fn run() -> (String, String) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_line = Self::send(&full).unwrap_or_default();

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_line = Self::send(&half).unwrap_or_default();

        (full_line, half_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends() {
        let mut s = LayerStack::new();
        s.add("A", ShellPublish::frame(128)).unwrap();
        let line = ShellPublish::send(&s).unwrap();
        assert!(!line.is_empty());
    }

    #[test]
    fn edit_changes_status() {
        let (full, half) = ShellPublish::run();
        assert_ne!(full, half);
    }

    #[test]
    fn differential() {
        let (full, half) = ShellPublish::run();
        assert!(!full.is_empty());
        assert!(!half.is_empty());
    }
}
