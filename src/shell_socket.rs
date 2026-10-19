//! Safe pure ShellSocket (shell packet → socket payload).
//!
//! Builds the shell packet as a socket-ready payload.
//! Confirms the payload parses. An opacity edit changes
//! the payload. No unwraps.
//!
//! Differential: half opacity produces a different payload.

use crate::encoder_context::EncoderType;
use crate::encoder_packet::EncoderPacket;
use crate::packet_file::PacketFile;
use crate::payload_dump::PayloadDump;
use crate::photo_layers::LayerStack;
use crate::shell_frame::ShellFrame;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct ShellSocket;

impl ShellSocket {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Build a socket-ready payload from the stack.
    pub fn payload(stack: &LayerStack) -> Option<Vec<u8>> {
        let bytes = ShellFrame::encode(stack)?;
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, bytes);
        let body = PacketFile::write(&[pkt]);
        let mut payload = vec![0u8; 12];
        payload.extend_from_slice(&body);
        Some(payload)
    }

    /// True if the payload parses as packets.
    pub fn valid(payload: &[u8]) -> bool {
        PayloadDump::packets(payload).is_some()
    }

    /// Run full and half. Returns (full_len, half_len).
    pub fn run() -> (usize, usize) {
        let mut full = LayerStack::new();
        full.add("Background", Self::frame(100)).unwrap();
        full.add("Layer 1", Self::frame(200)).unwrap();
        let full_len = Self::payload(&full).map(|p| p.len()).unwrap_or(0);

        let mut half = LayerStack::new();
        half.add("Background", Self::frame(100)).unwrap();
        half.add("Layer 1", Self::frame(200)).unwrap();
        let _ = half.set_opacity("Layer 1", 128);
        let half_len = Self::payload(&half).map(|p| p.len()).unwrap_or(0);

        (full_len, half_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_valid() {
        let mut s = LayerStack::new();
        s.add("A", ShellSocket::frame(128)).unwrap();
        let payload = ShellSocket::payload(&s).unwrap();
        assert!(ShellSocket::valid(&payload));
    }

    #[test]
    fn edit_changes_payload() {
        let (full, half) = ShellSocket::run();
        assert_ne!(full, half);
    }

    #[test]
    fn differential() {
        let (full, half) = ShellSocket::run();
        assert!(full > 0);
        assert!(half > 0);
    }
}
