//! Windows camera capture (Media Foundation).
//!
//! Calls the Windows camera API and feeds frames
//! into the Capture trait. Only compiles on Windows.
//!
//! The OS opens the camera. We use its API.

use crate::capture::Capture;
use crate::video_frame::VideoFrame;

#[cfg(windows)]
mod imp {
    use super::*;

    pub struct WindowsCapture {
        width: u32,
        height: u32,
        // Media Foundation source reader goes here.
        // IMFSourceReader, device enum, etc.
    }

    impl WindowsCapture {
        /// Open the default camera.
        pub fn open() -> Result<Self, String> {
            // TODO: CoInitialize, MFStartup, enumerate devices,
            // create IMFSourceReader, set media type.
            // Return the reader wrapped in Self.
            Err("Media Foundation init not yet wired".to_string())
        }

        pub fn open_index(index: u32) -> Result<Self, String> {
            let _ = index;
            Self::open()
        }
    }

    impl Capture for WindowsCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            // TODO: ReadSample, convert to Y800, return frame.
            None
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }
}

#[cfg(windows)]
pub use imp::WindowsCapture;

#[cfg(not(windows))]
pub struct WindowsCapture;

#[cfg(not(windows))]
impl WindowsCapture {
    pub fn open() -> Result<Self, String> {
        Err("Windows capture only available on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_on_this_os() {
        // This environment is not Windows.
        #[cfg(not(windows))]
        assert!(WindowsCapture::open().is_err());
    }
}
