//! macOS / iOS camera capture (AVFoundation).
//!
//! Calls the Apple camera API and feeds frames
//! into the Capture trait. Only compiles on Apple.
//!
//! The OS opens the camera. We use its API.

use crate::video_frame::VideoFrame;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod imp {
    use super::*;
    use crate::capture::Capture;

    pub struct AppleCapture {
        width: u32,
        height: u32,
        // AVCaptureSession goes here.
    }

    impl AppleCapture {
        pub fn open() -> Result<Self, String> {
            // TODO: AVCaptureDevice, AVCaptureSession, startRunning.
            Err("AVFoundation init not yet wired".to_string())
        }
    }

    impl Capture for AppleCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            None
        }
        fn width(&self) -> u32 { self.width }
        fn height(&self) -> u32 { self.height }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use imp::AppleCapture;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub struct AppleCapture;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
impl AppleCapture {
    pub fn open() -> Result<Self, String> {
        Err("Apple capture only available on macOS/iOS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_on_this_os() {
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        assert!(AppleCapture::open().is_err());
    }
}
