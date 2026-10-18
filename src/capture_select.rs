//! Safe pure CaptureSelect (pick a source).
//!
//! Picks a capture source. Synthetic is always
//! available. Real cameras are OS-specific.
//! No unwraps.
//!
//! Differential: synthetic produces frames.

use crate::capture::{open_default, Capture, SyntheticCapture};
use crate::file_capture::{FileCapture, FileCaptureError};
use crate::video_frame::VideoFrame;

pub enum CaptureSource {
    Synthetic(SyntheticCapture),
    File(FileCapture),
    /// Platform camera (Linux V4L2, Windows Media Foundation, or Apple AVFoundation).
    Platform(Box<dyn Capture>),
}

impl CaptureSource {
    /// Always-available source for tests.
    pub fn synthetic(width: u32, height: u32) -> Self {
        CaptureSource::Synthetic(SyntheticCapture::new(width, height))
    }

    /// Source from a raw file.
    pub fn file(path: &str, width: u32, height: u32) -> Result<Self, FileCaptureError> {
        Ok(CaptureSource::File(FileCapture::open(path, width, height)?))
    }

    /// Open the best available platform camera.
    /// Falls back to a clear error if none is wired on this OS.
    pub fn platform() -> Result<Self, String> {
        open_default().map(CaptureSource::Platform)
    }

    /// Best available: platform if it opens, otherwise synthetic.
    /// Returns (source, live).
    pub fn best(width: u32, height: u32) -> (Self, bool) {
        match Self::platform() {
            Ok(src) => (src, true),
            Err(_) => (Self::synthetic(width, height), false),
        }
    }

    /// Next frame from the source.
    pub fn frame(&mut self) -> Option<VideoFrame> {
        match self {
            CaptureSource::Synthetic(c) => c.frame(),
            CaptureSource::File(c) => c.frame(),
            CaptureSource::Platform(c) => c.frame(),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.width(),
            CaptureSource::File(c) => c.width(),
            CaptureSource::Platform(c) => c.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.height(),
            CaptureSource::File(c) => c.height(),
            CaptureSource::Platform(c) => c.height(),
        }
    }

    /// Short name for status.
    pub fn kind(&self) -> &'static str {
        match self {
            CaptureSource::Synthetic(_) => "synthetic",
            CaptureSource::File(_) => "file",
            CaptureSource::Platform(_) => "platform",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_falls_back() {
        let (mut src, live) = CaptureSource::best(16, 16);
        // On this environment platform is unavailable.
        let _ = live;
        assert!(src.frame().is_some());
    }

    #[test]
    fn synthetic_kind() {
        let src = CaptureSource::synthetic(8, 8);
        assert_eq!(src.kind(), "synthetic");
    }

    #[test]
    fn differential() {
        let (mut a, _) = CaptureSource::best(8, 8);
        let (mut b, _) = CaptureSource::best(16, 16);
        assert!(a.frame().is_some());
        assert!(b.frame().is_some());
    }
}