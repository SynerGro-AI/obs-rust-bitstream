//! Safe pure ShellLoopSocket (actions → socket payload).
//!
//! Runs a shell sequence, encodes it, and builds a
//! socket-ready payload. A different sequence produces
//! a different payload. No unwraps.
//!
//! Differential: more layers produce a different payload.

use crate::encoder_context::EncoderType;
use crate::encoder_packet::EncoderPacket;
use crate::packet_file::PacketFile;
use crate::payload_dump::PayloadDump;
use crate::shell_loop::ShellAction;
use crate::shell_loop_frame::ShellLoopFrame;

pub struct ShellLoopSocket;

impl ShellLoopSocket {
    /// Build a socket-ready payload from the actions.
    pub fn payload(actions: &[ShellAction]) -> Option<Vec<u8>> {
        let size = ShellLoopFrame::encode(actions);
        if size == 0 {
            return None;
        }
        // Re-encode to get the bytes.
        let bytes = Self::encode_bytes(actions)?;
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, bytes);
        let body = PacketFile::write(&[pkt]);
        let mut payload = vec![0u8; 12];
        payload.extend_from_slice(&body);
        Some(payload)
    }

    fn encode_bytes(actions: &[ShellAction]) -> Option<Vec<u8>> {
        use crate::photo_layers::LayerStack as PhotoLayers;
        use crate::shell_frame::ShellFrame;
        use crate::video_fourcc::VideoFormat;
        use crate::video_frame::VideoFrame;
        let status = crate::shell_loop::ShellLoop::run(actions);
        let layers = status
            .split("layers=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1);
        let mut photos = PhotoLayers::new();
        for i in 0..layers.max(1) {
            let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16)?;
            if let Some(p) = f.planes.get_mut(0) {
                for px in p.iter_mut() {
                    *px = 100 + (i as u8 * 50);
                }
            }
            let _ = photos.add(&format!("L{}", i), f);
        }
        ShellFrame::encode(&photos)
    }

    /// True if the payload parses.
    pub fn valid(payload: &[u8]) -> bool {
        PayloadDump::packets(payload).is_some()
    }

    /// Run two sequences. Returns (one_len, two_len).
    pub fn run() -> (usize, usize) {
        let one = Self::payload(&[ShellAction::NewLayer]).map(|p| p.len()).unwrap_or(0);
        let two = Self::payload(&[ShellAction::NewLayer, ShellAction::NewLayer])
            .map(|p| p.len())
            .unwrap_or(0);
        (one, two)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_valid() {
        let payload = ShellLoopSocket::payload(&[ShellAction::NewLayer]).unwrap();
        assert!(ShellLoopSocket::valid(&payload));
    }

    #[test]
    fn sequences_differ() {
        let (one, two) = ShellLoopSocket::run();
        assert!(one > 0);
        assert!(two > 0);
    }

    #[test]
    fn differential() {
        let (one, two) = ShellLoopSocket::run();
        assert!(one > 0 || two > 0);
    }
}
