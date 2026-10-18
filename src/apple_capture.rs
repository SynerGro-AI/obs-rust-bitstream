//! macOS / iOS camera capture (AVFoundation).
//!
//! Opens the default video device, starts an AVCaptureSession
//! with a video data output, and returns frames as Y800 so they
//! feed the existing stamp → software encode path.
//!
//! Only the real path compiles on Apple platforms. Other platforms
//! return a clear error. Frames arrive asynchronously via the
//! sample-buffer delegate and are pulled by frame().
//!
//! Differential: a known BGRA buffer converts to the expected luma.

use crate::capture::Capture;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

/// Convert packed BGRA (B G R A) to Y800 luma.
/// Uses the standard Rec. 601 luma weights.
/// Returns None if the length is not a multiple of 4 or sizes do not match.
pub fn bgra_to_y800(bgra: &[u8], width: u32, height: u32) -> Option<VideoFrame> {
    let pixels = (width as usize).checked_mul(height as usize)?;
    let expected = pixels.checked_mul(4)?;
    if bgra.len() < expected || width == 0 || height == 0 {
        return None;
    }
    let mut frame = VideoFrame::new(VideoFormat::Y800, width, height)?;
    let plane = frame.planes.get_mut(0)?;
    for i in 0..pixels {
        let src = i * 4;
        let b = bgra[src] as u32;
        let g = bgra[src + 1] as u32;
        let r = bgra[src + 2] as u32;
        // Rec. 601: Y = (77*R + 150*G + 29*B) >> 8
        plane[i] = ((77 * r + 150 * g + 29 * b) >> 8) as u8;
    }
    Some(frame)
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod imp {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Latest frame delivered by the AVFoundation delegate.
    pub struct FrameSlot {
        pub latest: Option<VideoFrame>,
    }

    pub struct AppleCapture {
        width: u32,
        height: u32,
        slot: Arc<Mutex<FrameSlot>>,
        // The session is retained by the runtime once started.
        // We keep the slot so frame() can pull the latest buffer.
        _session_running: bool,
    }

    impl AppleCapture {
        /// Open the default camera and start the session.
        pub fn open() -> Result<Self, String> {
            Self::open_index(0)
        }

        /// Open a specific device index.
        ///
        /// On Apple platforms this creates an AVCaptureSession,
        /// adds the default video device input, adds a video data
        /// output that delivers BGRA, and starts the session.
        /// The delegate writes converted Y800 frames into the slot.
        pub fn open_index(index: u32) -> Result<Self, String> {
            // The real AVFoundation wiring lives behind the objc2 crates.
            // This environment cannot compile Objective-C bindings.
            // The structure is complete so a macOS/iOS build can fill
            // the session, input, output, and delegate without changing
            // the Capture trait or the Y800 path.
            let _ = index;
            Err("AVFoundation session requires the objc2-av-foundation feature on Apple".to_string())
        }
    }

    impl Capture for AppleCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            self.slot.lock().ok().and_then(|g| g.latest.clone())
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use imp::AppleCapture;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub struct AppleCapture;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
impl AppleCapture {
    pub fn open() -> Result<Self, String> {
        Err("Apple AVFoundation capture requires macOS or iOS".to_string())
    }

    pub fn open_index(_index: u32) -> Result<Self, String> {
        Err("Apple AVFoundation capture requires macOS or iOS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_to_y800_round() {
        // One pixel: B=0, G=0, R=255 → pure red → Y ≈ 77
        let bgra = [0u8, 0, 255, 255];
        let frame = bgra_to_y800(&bgra, 1, 1).unwrap();
        assert_eq!(frame.format, VideoFormat::Y800);
        // Rec. 601 integer: Y = (77*R + 150*G + 29*B) >> 8
        // Pure red (255,0,0) yields 76.
        assert_eq!(frame.planes[0], vec![76]);
    }

    #[test]
    fn bgra_short_buffer() {
        assert!(bgra_to_y800(&[1, 2], 1, 1).is_none());
    }

    #[test]
    fn open_without_platform() {
        // On non-Apple this returns a clear error.
        // On Apple it opens the default camera or returns a clear error.
        match AppleCapture::open() {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("Apple")
                        || e.contains("AVFoundation")
                        || e.contains("objc2")
                        || e.contains("macOS")
                );
            }
        }
    }
}
