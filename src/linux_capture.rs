//! Linux camera capture (V4L2).
//!
//! Calls the Linux camera API and feeds frames
//! into the Capture trait. Only compiles on Linux.
//!
//! The OS opens the camera. We use its API.

use crate::video_frame::VideoFrame;

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use crate::capture::Capture;

    pub struct LinuxCapture {
        width: u32,
        height: u32,
        // V4L2 file descriptor goes here.
    }

    impl LinuxCapture {
        pub fn open() -> Result<Self, String> {
            // TODO: open /dev/video0, VIDIOC_S_FMT, mmap.
            Err("V4L2 init not yet wired".to_string())
        }
    }

    impl Capture for LinuxCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            None
        }
        fn width(&self) -> u32 { self.width }
        fn height(&self) -> u32 { self.height }
    }
}

#[cfg(target_os = "linux")]
pub use imp::LinuxCapture;

#[cfg(not(target_os = "linux"))]
pub struct LinuxCapture;

#[cfg(not(target_os = "linux"))]
impl LinuxCapture {
    pub fn open() -> Result<Self, String> {
        Err("Linux capture only available on Linux".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_exists() {
        // On Linux the real path is cfg-gated.
        // On other OSes open returns an error.
        let _ = LinuxCapture::open();
    }
}
