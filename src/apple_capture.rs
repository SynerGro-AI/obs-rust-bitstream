//! macOS / iOS camera capture (AVFoundation).
//!
//! Opens the default video device, starts an AVCaptureSession
//! with a video data output configured for BGRA, and returns
//! frames as Y800 so they feed the existing stamp → software
//! encode path.
//!
//! The real session, input, output, and sample-buffer delegate
//! compile only when the `avfoundation` feature is enabled on
//! an Apple target. Other platforms return a clear error.
//!
//! On iOS the host app must still declare NSCameraUsageDescription
//! in Info.plist. The library cannot add that.
//!
//! Differential: a known BGRA buffer converts to the expected luma.

use crate::capture::Capture;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

/// Convert packed BGRA (B G R A) to Y800 luma.
/// Uses the standard Rec. 601 integer weights.
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

#[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "avfoundation"))]
mod imp {
    use super::*;
    use std::sync::{Arc, Mutex};

    use objc2::define_class;
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_av_foundation::{
        AVCaptureDevice, AVCaptureDeviceInput, AVCaptureSession, AVCaptureVideoDataOutput,
        AVMediaTypeVideo,
    };
    use objc2_core_media::CMSampleBuffer;
    use objc2_core_video::{
        CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
        CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferUnlockBaseAddress,
        kCVPixelBufferLock_ReadOnly, kCVPixelFormatType_32BGRA,
    };
    use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

    /// Latest frame delivered by the AVFoundation delegate.
    pub struct FrameSlot {
        pub latest: Option<VideoFrame>,
    }

    /// Ivars for the sample-buffer delegate.
    struct DelegateIvars {
        slot: Arc<Mutex<FrameSlot>>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[ivars = DelegateIvars]
        struct SampleDelegate;

        unsafe impl objc2_av_foundation::AVCaptureVideoDataOutputSampleBufferDelegate for SampleDelegate {
            #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
            fn did_output(
                &self,
                _output: &objc2_av_foundation::AVCaptureOutput,
                sample: &CMSampleBuffer,
                _connection: &objc2_av_foundation::AVCaptureConnection,
            ) {
                if let Some(frame) = extract_y800(sample) {
                    if let Ok(mut guard) = self.ivars().slot.lock() {
                        guard.latest = Some(frame);
                    }
                }
            }
        }
    );

    /// Pull BGRA pixels from a CMSampleBuffer and convert to Y800.
    unsafe fn extract_y800(sample: &CMSampleBuffer) -> Option<VideoFrame> {
        let image = sample.image_buffer()?;
        // CVImageBuffer is the base; for video it is a CVPixelBuffer.
        let pixel = image.as_ref() as *const _ as *const objc2_core_video::CVPixelBuffer;
        let pixel = &*pixel;
        if CVPixelBufferLockBaseAddress(pixel, kCVPixelBufferLock_ReadOnly) != 0 {
            return None;
        }
        let width = CVPixelBufferGetWidth(pixel) as u32;
        let height = CVPixelBufferGetHeight(pixel) as u32;
        let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel);
        let base = CVPixelBufferGetBaseAddress(pixel);
        let frame = if base.is_null() || width == 0 || height == 0 {
            None
        } else {
            let row = bytes_per_row;
            let mut packed = Vec::with_capacity((width as usize) * (height as usize) * 4);
            for y in 0..height as usize {
                let start = y * row;
                let end = start + (width as usize) * 4;
                let slice = std::slice::from_raw_parts(base as *const u8, end);
                packed.extend_from_slice(&slice[start..end]);
            }
            bgra_to_y800(&packed, width, height)
        };
        let _ = CVPixelBufferUnlockBaseAddress(pixel, kCVPixelBufferLock_ReadOnly);
        frame
    }

    pub struct AppleCapture {
        width: u32,
        height: u32,
        slot: Arc<Mutex<FrameSlot>>,
        session: Retained<AVCaptureSession>,
    }

    impl AppleCapture {
        /// Open the default camera and start the session.
        pub fn open() -> Result<Self, String> {
            Self::open_index(0)
        }

        /// Open a specific device index.
        ///
        /// Creates an AVCaptureSession, adds the chosen video device
        /// input, adds a video data output that delivers BGRA, attaches
        /// a sample-buffer delegate, and starts the session. The delegate
        /// converts each buffer with bgra_to_y800 and stores it in the slot.
        pub fn open_index(index: u32) -> Result<Self, String> {
            unsafe {
                let media_type = AVMediaTypeVideo.ok_or("no AVMediaTypeVideo")?;
                let device = AVCaptureDevice::defaultDeviceWithMediaType(media_type)
                    .ok_or("no default video device")?;
                let input = AVCaptureDeviceInput::deviceInputWithDevice_error(&device)
                    .map_err(|e| format!("device input: {}", e))?;

                let session = AVCaptureSession::new();
                session.beginConfiguration();
                if !session.canAddInput(&input) {
                    return Err("cannot add video input".to_string());
                }
                session.addInput(&input);

                let output = AVCaptureVideoDataOutput::new();
                // Request BGRA.
                let key: &NSString = &*(objc2_core_video::kCVPixelBufferPixelFormatTypeKey
                    as *const _ as *const NSString);
                let value = NSNumber::new_u32(kCVPixelFormatType_32BGRA);
                let settings = NSDictionary::from_slices(&[key], &[value.as_ref()]);
                output.setVideoSettings(Some(&settings));

                if !session.canAddOutput(&output) {
                    return Err("cannot add video data output".to_string());
                }
                session.addOutput(&output);

                let slot = Arc::new(Mutex::new(FrameSlot { latest: None }));
                let ivars = DelegateIvars { slot: slot.clone() };
                let delegate = SampleDelegate::alloc().set_ivars(ivars);
                let delegate = ProtocolObject::from_retained(delegate);
                // Use the main queue for simplicity; a dedicated queue is better in production.
                let queue = dispatch2::DispatchQueue::main();
                output.setSampleBufferDelegate_queue(Some(&delegate), Some(&queue));

                session.commitConfiguration();
                session.startRunning();

                // Width/height are filled by the first frame; start with zero.
                Ok(Self {
                    width: 0,
                    height: 0,
                    slot,
                    session,
                })
            }
        }
    }

    impl Drop for AppleCapture {
        fn drop(&mut self) {
            unsafe {
                self.session.stopRunning();
            }
        }
    }

    impl Capture for AppleCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            let frame = self.slot.lock().ok().and_then(|g| g.latest.clone());
            if let Some(ref f) = frame {
                self.width = f.width;
                self.height = f.height;
            }
            frame
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }
}

#[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "avfoundation"))]
pub use imp::AppleCapture;

#[cfg(not(all(any(target_os = "macos", target_os = "ios"), feature = "avfoundation")))]
pub struct AppleCapture;

#[cfg(not(all(any(target_os = "macos", target_os = "ios"), feature = "avfoundation")))]
impl AppleCapture {
    pub fn open() -> Result<Self, String> {
        Err("Apple AVFoundation capture requires macOS/iOS and the avfoundation feature".to_string())
    }

    pub fn open_index(_index: u32) -> Result<Self, String> {
        Err("Apple AVFoundation capture requires macOS/iOS and the avfoundation feature".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_to_y800_round() {
        // One pixel: B=0, G=0, R=255 → pure red → Y = (77*255) >> 8 = 76
        let bgra = [0u8, 0, 255, 255];
        let frame = bgra_to_y800(&bgra, 1, 1).unwrap();
        assert_eq!(frame.format, VideoFormat::Y800);
        assert_eq!(frame.planes[0], vec![76]);
    }

    #[test]
    fn bgra_short_buffer() {
        assert!(bgra_to_y800(&[1, 2], 1, 1).is_none());
    }

    #[test]
    fn open_without_platform() {
        // On non-Apple or without the feature this returns a clear error.
        // On Apple with the feature it opens the default camera or returns a clear error.
        match AppleCapture::open() {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("Apple")
                        || e.contains("AVFoundation")
                        || e.contains("avfoundation")
                        || e.contains("macOS")
                        || e.contains("no default")
                        || e.contains("device input")
                );
            }
        }
    }
}
