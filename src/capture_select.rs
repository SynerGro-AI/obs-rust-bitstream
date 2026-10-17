//! Safe pure CaptureSelect (pick a source).
//!
//! Picks a capture source. Synthetic is always
//! available. Real cameras are OS-specific.
//! No unwraps.
//!
//! Differential: synthetic produces frames.

use crate::capture::{Capture, SyntheticCapture};
use crate::file_capture::{FileCapture, FileCaptureError};
use crate::video_frame::VideoFrame;

pub enum CaptureSource {
    Synthetic(SyntheticCapture),
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

    /// Next frame from the source.
    pub fn frame(&mut self) -> Option<VideoFrame> {
        match self {
            CaptureSource::Synthetic(c) => c.frame(),
            CaptureSource::File(c) => c.frame(),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.width(),
            CaptureSource::File(c) => c.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.height(),
            CaptureSource::File(c) => c.height(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut src = CaptureSource::synthetic(8, 8);
        let a = src.frame().unwrap();
        let b = src.frame().unwrap();
        assert_ne!(a.planes[0][0], b.planes[0][0]);
        assert_eq!(src.width(), 8);
        let path = "/tmp/sel_frames.raw";
        std::fs::write(path, &[7u8; 16]).unwrap();
        let mut file_src = CaptureSource::file(path, 4, 4).unwrap();
        let f = file_src.frame().unwrap();
        assert_eq!(f.planes[0][0], 7);
    }
}
