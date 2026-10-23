//! Safe pure FileCapture (read frames from a file).
//!
//! Reads raw Y800 frames from a file. No unwraps.
//!
//! Differential: a known file produces a frame.

use crate::capture::Capture;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum FileCaptureError {
    Io,
    BadSize,
}

pub struct FileCapture {
    file: File,
    width: u32,
    height: u32,
}

impl FileCapture {
    pub fn open(path: &str, width: u32, height: u32) -> Result<Self, FileCaptureError> {
        if width == 0 || height == 0 {
            return Err(FileCaptureError::BadSize);
        }
        let file = File::open(path).map_err(|_| FileCaptureError::Io)?;
        Ok(Self {
            file,
            width,
            height,
        })
    }
}

impl Capture for FileCapture {
    fn frame(&mut self) -> Option<VideoFrame> {
        let size = (self.width * self.height) as usize;
        let mut buf = vec![0u8; size];
        self.file.read_exact(&mut buf).ok()?;
        let mut f = VideoFrame::new(VideoFormat::Y800, self.width, self.height)?;
        f.planes[0].copy_from_slice(&buf);
        Some(f)
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn differential() {
        let path = "/tmp/test_frames.raw";
        {
            let mut f = File::create(path).unwrap();
            f.write_all(&[10u8; 16]).unwrap();
            f.write_all(&[20u8; 16]).unwrap();
        }
        let mut cap = FileCapture::open(path, 4, 4).unwrap();
        let a = cap.frame().unwrap();
        assert_eq!(a.planes[0][0], 10);
        let b = cap.frame().unwrap();
        assert_eq!(b.planes[0][0], 20);
        assert!(cap.frame().is_none());
    }
}
