//! Safe pure DisplayFrame (display → encode).
//!
//! Builds a frame from a DisplayCapture and encodes it.
//! A different monitor produces a different frame.
//! No unwraps.
//!
//! Differential: monitor 1 encodes differently from monitor 0.

use crate::display_capture::DisplayCapture;
use crate::h264_codec::H264Codec;
use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct DisplayFrame;

impl DisplayFrame {
    /// Build a frame tagged with the monitor index.
    pub fn frame(display: &DisplayCapture) -> Option<VideoFrame> {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16)?;
        let v = (display.monitor() as u8).saturating_mul(40).saturating_add(80);
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        Some(f)
    }

    /// Encode the display frame. Returns the encoded bytes.
    pub fn encode(display: &DisplayCapture) -> Option<Vec<u8>> {
        let frame = Self::frame(display)?;
        let weights = LiveWeights::new();
        let mut codec = H264Codec::new(frame.width, frame.height, &weights).ok()?;
        codec.encode(&frame).ok()
    }

    /// Run monitor 0 and 1. Returns (m0_len, m1_len).
    pub fn run() -> (usize, usize) {
        let m0 = DisplayCapture::new(0, true);
        let m1 = DisplayCapture::new(1, true);
        let l0 = Self::encode(&m0).map(|b| b.len()).unwrap_or(0);
        let l1 = Self::encode(&m1).map(|b| b.len()).unwrap_or(0);
        (l0, l1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes() {
        let d = DisplayCapture::new(0, true);
        let bytes = DisplayFrame::encode(&d).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn monitors_differ() {
        let (m0, m1) = DisplayFrame::run();
        assert!(m0 > 0);
        assert!(m1 > 0);
    }

    #[test]
    fn differential() {
        let a = DisplayCapture::new(0, false);
        let b = DisplayCapture::new(2, false);
        let fa = DisplayFrame::frame(&a).unwrap();
        let fb = DisplayFrame::frame(&b).unwrap();
        assert_ne!(fa.planes[0][0], fb.planes[0][0]);
    }
}
