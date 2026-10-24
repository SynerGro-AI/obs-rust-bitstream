//! Safe pure Capture (frame source core).
//!
//! A trait for anything that produces frames.
//! SyntheticCapture generates a moving pattern.
//! No unwraps. No platform code.
//!
//! Differential: two frames differ.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub trait Capture {
    fn frame(&mut self) -> Option<VideoFrame>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

/// Open the default camera for this platform.
/// On Linux this opens /dev/video0 via V4L2.
/// On Windows this opens the default Media Foundation device.
/// On macOS/iOS this opens the default AVFoundation device.
/// Other platforms return a clear error until their backends are filled.
pub fn open_default() -> Result<Box<dyn Capture>, String> {
    #[cfg(target_os = "linux")]
    {
        crate::linux_capture::LinuxCapture::open().map(|c| Box::new(c) as Box<dyn Capture>)
    }
    #[cfg(windows)]
    {
        crate::windows_capture::WindowsCapture::open().map(|c| Box::new(c) as Box<dyn Capture>)
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        crate::apple_capture::AppleCapture::open().map(|c| Box::new(c) as Box<dyn Capture>)
    }
    #[cfg(not(any(target_os = "linux", windows, target_os = "macos", target_os = "ios")))]
    {
        Err("default camera not yet wired on this platform".to_string())
    }
}

#[derive(Clone, Debug)]
pub struct SyntheticCapture {
    width: u32,
    height: u32,
    frame: u32,
}

impl SyntheticCapture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frame: 0,
        }
    }
}

impl Capture for SyntheticCapture {
    fn frame(&mut self) -> Option<VideoFrame> {
        let mut f = VideoFrame::new(VideoFormat::Y800, self.width, self.height)?;
        let shift = (self.frame % self.width) as usize;
        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                let v = ((x + shift + y) % 256) as u8;
                f.planes[0][y * self.width as usize + x] = v;
            }
        }
        self.frame += 1;
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

    #[test]
    fn differential() {
        let mut c = SyntheticCapture::new(8, 8);
        let a = c.frame().unwrap();
        let b = c.frame().unwrap();
        assert_ne!(a.planes[0][0], b.planes[0][0]);
        assert_eq!(c.width(), 8);
        assert_eq!(c.height(), 8);
    }

    #[test]
    fn open_default_exists() {
        // Returns a clear error or a working camera.
        let _ = open_default();
    }
}
