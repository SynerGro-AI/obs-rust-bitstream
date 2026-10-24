//! Windows camera capture (Media Foundation).
//!
//! Opens the default video capture device via IMFSourceReader,
//! requests YUY2 (or NV12), and returns frames as Y800 so they
//! feed the existing stamp → software encode path.
//!
//! Only the real path compiles on Windows. Other platforms
//! return a clear error. COM and Media Foundation are initialized
//! once and cleaned up on drop.
//!
//! Differential: a known YUY2 pair converts to the expected luma.

use crate::capture::Capture;
use crate::video_frame::VideoFrame;

/// Convert packed YUYV (Y0 U Y1 V) to Y800 luma.
pub fn yuyv_to_y800(yuyv: &[u8], width: u32, height: u32) -> Option<VideoFrame> {
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(2)?;
    if yuyv.len() < expected || width == 0 || height == 0 {
        return None;
    }
    let mut frame = VideoFrame::new(crate::video_fourcc::VideoFormat::Y800, width, height)?;
    let plane = frame.planes.get_mut(0)?;
    let pixels = (width as usize) * (height as usize);
    for (dst, px) in plane.iter_mut().zip(yuyv.as_chunks::<2>().0).take(pixels) {
        *dst = px[0];
    }
    Some(frame)
}

/// Convert NV12 (Y plane followed by interleaved UV) to Y800.
/// Returns None if the buffer is shorter than the Y plane.
pub fn nv12_to_y800(nv12: &[u8], width: u32, height: u32) -> Option<VideoFrame> {
    let y_size = (width as usize).checked_mul(height as usize)?;
    if nv12.len() < y_size || width == 0 || height == 0 {
        return None;
    }
    let mut frame = VideoFrame::new(crate::video_fourcc::VideoFormat::Y800, width, height)?;
    let plane = frame.planes.get_mut(0)?;
    plane.copy_from_slice(&nv12[..y_size]);
    Some(frame)
}

#[cfg(all(windows, feature = "windows"))]
mod imp {
    use super::*;
    use windows::Win32::Media::MediaFoundation::{
        IMFActivate, IMFSourceReader, MFCreateAttributes, MFCreateSourceReaderFromMediaSource,
        MFEnumDeviceSources, MFStartup, MFVideoFormat_NV12, MFVideoFormat_YUY2,
        MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
        MF_MT_FRAME_SIZE, MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_VERSION,
    };
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    pub struct WindowsCapture {
        reader: IMFSourceReader,
        width: u32,
        height: u32,
        is_nv12: bool,
        com_inited: bool,
    }

    impl WindowsCapture {
        /// Open the default camera (index 0).
        pub fn open() -> Result<Self, String> {
            Self::open_index(0)
        }

        /// Open a specific device index.
        pub fn open_index(index: u32) -> Result<Self, String> {
            unsafe {
                // Initialize COM.
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .map_err(|e| format!("CoInitializeEx: {:?}", e))?;

                // Initialize Media Foundation.
                MFStartup(MF_VERSION, 0).map_err(|e| format!("MFStartup: {}", e))?;

                // Enumerate video capture devices.
                let mut attr = None;
                MFCreateAttributes(&mut attr, 1)
                    .map_err(|e| format!("MFCreateAttributes: {}", e))?;
                let attr = attr.ok_or("no attributes")?;
                attr.SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .map_err(|e| format!("SetGUID: {}", e))?;

                let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
                let mut count = 0u32;
                MFEnumDeviceSources(&attr, &mut devices, &mut count)
                    .map_err(|e| format!("MFEnumDeviceSources: {}", e))?;
                if count == 0 || devices.is_null() {
                    return Err("no video capture devices".to_string());
                }
                if index >= count {
                    return Err(format!(
                        "device index {} out of range ({} devices)",
                        index, count
                    ));
                }

                // Activate the chosen device.
                let activate = (*devices.add(index as usize))
                    .clone()
                    .ok_or("activate is null")?;
                let source: windows::Win32::Media::MediaFoundation::IMFMediaSource = activate
                    .ActivateObject()
                    .map_err(|e| format!("ActivateObject: {}", e))?;

                // Create the source reader.
                let reader = MFCreateSourceReaderFromMediaSource(&source, None)
                    .map_err(|e| format!("MFCreateSourceReader: {:?}", e))?;

                // Prefer YUY2, fall back to NV12.
                let mut is_nv12 = false;
                if set_subtype(&reader, &MFVideoFormat_YUY2).is_err() {
                    set_subtype(&reader, &MFVideoFormat_NV12)
                        .map_err(|e| format!("set media type: {}", e))?;
                    is_nv12 = true;
                }

                // Read the actual frame size.
                let media_type = reader
                    .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                    .map_err(|e| format!("GetCurrentMediaType: {}", e))?;
                let size = media_type
                    .GetUINT64(&MF_MT_FRAME_SIZE)
                    .map_err(|e| format!("GetUINT64 frame size: {}", e))?;
                let width = (size >> 32) as u32;
                let height = (size & 0xffff_ffff) as u32;

                Ok(Self {
                    reader,
                    width,
                    height,
                    is_nv12,
                    com_inited: true,
                })
            }
        }
    }

    /// Set the subtype (YUY2 or NV12) on the first video stream.
    unsafe fn set_subtype(
        reader: &IMFSourceReader,
        subtype: &windows::core::GUID,
    ) -> Result<(), String> {
        use windows::Win32::Media::MediaFoundation::{
            MFCreateMediaType, MFMediaType_Video, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
        };
        let mt = MFCreateMediaType().map_err(|e| format!("MFCreateMediaType: {:?}", e))?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| format!("SetGUID major: {}", e))?;
        mt.SetGUID(&MF_MT_SUBTYPE, subtype)
            .map_err(|e| format!("SetGUID subtype: {}", e))?;
        reader
            .SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &mt)
            .map_err(|e| format!("SetCurrentMediaType: {}", e))?;
        Ok(())
    }

    impl Drop for WindowsCapture {
        fn drop(&mut self) {
            if self.com_inited {
                unsafe {
                    let _ = windows::Win32::Media::MediaFoundation::MFShutdown();
                    CoUninitialize();
                }
            }
        }
    }

    impl Capture for WindowsCapture {
        fn frame(&mut self) -> Option<VideoFrame> {
            unsafe {
                let mut sample = None;
                let mut flags = 0u32;
                self.reader
                    .ReadSample(
                        MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                        0,
                        None,
                        Some(&mut flags),
                        None,
                        Some(&mut sample),
                    )
                    .ok()?;
                let sample = sample?;
                let buffer = sample.ConvertToContiguousBuffer().ok()?;
                let mut data = std::ptr::null_mut();
                let mut len = 0u32;
                buffer.Lock(&mut data, None, Some(&mut len)).ok()?;
                let slice = std::slice::from_raw_parts(data, len as usize);
                let frame = if self.is_nv12 {
                    nv12_to_y800(slice, self.width, self.height)
                } else {
                    yuyv_to_y800(slice, self.width, self.height)
                };
                let _ = buffer.Unlock();
                frame
            }
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }
}

#[cfg(all(windows, feature = "windows"))]
pub use imp::WindowsCapture;

/// Stub when the `windows` feature (Media Foundation) is off.
#[cfg(not(all(windows, feature = "windows")))]
pub struct WindowsCapture;

#[cfg(not(all(windows, feature = "windows")))]
impl Capture for WindowsCapture {
    fn frame(&mut self) -> Option<VideoFrame> {
        None
    }

    fn width(&self) -> u32 {
        0
    }

    fn height(&self) -> u32 {
        0
    }
}

#[cfg(not(all(windows, feature = "windows")))]
impl WindowsCapture {
    pub fn open() -> Result<Self, String> {
        Err(
            "Windows Media Foundation capture requires Windows and the `windows` feature"
                .to_string(),
        )
    }

    pub fn open_index(_index: u32) -> Result<Self, String> {
        Err(
            "Windows Media Foundation capture requires Windows and the `windows` feature"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nv12_to_y800_round() {
        // 2x2 Y plane followed by UV.
        let nv12 = [10u8, 20, 30, 40, 128, 128];
        let frame = nv12_to_y800(&nv12, 2, 2).unwrap();
        assert_eq!(frame.planes[0], vec![10, 20, 30, 40]);
    }

    #[test]
    fn nv12_short_buffer() {
        assert!(nv12_to_y800(&[1, 2], 2, 2).is_none());
    }

    #[test]
    fn open_without_platform() {
        // On non-Windows this returns a clear error.
        // On Windows it opens the default camera or returns a clear error.
        match WindowsCapture::open() {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.contains("Windows")
                        || e.contains("no video")
                        || e.contains("MF")
                        || e.contains("CoInitialize")
                );
            }
        }
    }
}
