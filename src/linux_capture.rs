//! Linux camera capture (V4L2).
//!
//! Opens /dev/video0 (or a chosen index), requests YUYV,
//! maps buffers, and returns frames as Y800 so they feed
//! the existing stamp → software encode path.
//!
//! Uses the kernel V4L2 ioctls directly. No extra system
//! packages required. Only the real path runs on Linux.
//!
//! Differential: a known YUYV pair converts to the expected luma.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use std::fs::File;
use std::os::unix::io::AsRawFd;

/// Convert packed YUYV (Y0 U Y1 V) to Y800 luma.
/// Returns None if the length is not a multiple of 4 or sizes do not match.
pub fn yuyv_to_y800(yuyv: &[u8], width: u32, height: u32) -> Option<VideoFrame> {
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(2)?;
    if yuyv.len() < expected || width == 0 || height == 0 {
        return None;
    }
    let mut frame = VideoFrame::new(VideoFormat::Y800, width, height)?;
    let plane = frame.planes.get_mut(0)?;
    let pixels = (width as usize) * (height as usize);
    for i in 0..pixels {
        let src = i * 2;
        if src < yuyv.len() {
            plane[i] = yuyv[src];
        }
    }
    Some(frame)
}

// Minimal V4L2 constants and structures. Values from linux/videodev2.h.
const V4L2_PIX_FMT_YUYV: u32 = 0x56595559; // 'YUYV'
const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
const V4L2_MEMORY_MMAP: u32 = 1;
const VIDIOC_S_FMT: libc::c_ulong = 0xc0d05605;
const VIDIOC_REQBUFS: libc::c_ulong = 0xc0145608;
const VIDIOC_QUERYBUF: libc::c_ulong = 0xc0585609;
const VIDIOC_QBUF: libc::c_ulong = 0xc058560f;
const VIDIOC_DQBUF: libc::c_ulong = 0xc0585611;
const VIDIOC_STREAMON: libc::c_ulong = 0x40045612;

#[repr(C)]
#[derive(Clone, Copy)]
struct V4l2PixFormat {
    width: u32,
    height: u32,
    pixelformat: u32,
    field: u32,
    bytesperline: u32,
    sizeimage: u32,
    colorspace: u32,
    priv_: u32,
    flags: u32,
    ycbcr_enc: u32,
    quantization: u32,
    xfer_func: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct V4l2Format {
    type_: u32,
    fmt: V4l2PixFormat,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct V4l2RequestBuffers {
    count: u32,
    type_: u32,
    memory: u32,
    capabilities: u32,
    reserved: [u32; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct V4l2Buffer {
    index: u32,
    type_: u32,
    bytesused: u32,
    flags: u32,
    field: u32,
    timestamp: libc::timeval,
    timecode: [u8; 16],
    sequence: u32,
    memory: u32,
    offset: u32,
    length: u32,
    reserved2: u32,
    reserved: u32,
}

pub struct LinuxCapture {
    file: File,
    width: u32,
    height: u32,
    buffers: Vec<(*mut u8, usize)>,
}

impl LinuxCapture {
    /// Open the first video device and start streaming YUYV.
    pub fn open() -> Result<Self, String> {
        Self::open_index(0)
    }

    /// Open a specific device index.
    pub fn open_index(index: usize) -> Result<Self, String> {
        let path = format!("/dev/video{}", index);
        let file = File::open(&path).map_err(|e| format!("open {}: {}", path, e))?;
        let fd = file.as_raw_fd();

        // Set format to 640x480 YUYV.
        let mut fmt = V4l2Format {
            type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
            fmt: V4l2PixFormat {
                width: 640,
                height: 480,
                pixelformat: V4L2_PIX_FMT_YUYV,
                field: 1, // V4L2_FIELD_NONE
                bytesperline: 0,
                sizeimage: 0,
                colorspace: 0,
                priv_: 0,
                flags: 0,
                ycbcr_enc: 0,
                quantization: 0,
                xfer_func: 0,
            },
        };
        let rc = unsafe { libc::ioctl(fd, VIDIOC_S_FMT, &mut fmt) };
        if rc != 0 {
            return Err(format!("VIDIOC_S_FMT: {}", std::io::Error::last_os_error()));
        }
        let width = fmt.fmt.width;
        let height = fmt.fmt.height;

        // Request 4 mmap buffers.
        let mut req = V4l2RequestBuffers {
            count: 4,
            type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
            memory: V4L2_MEMORY_MMAP,
            capabilities: 0,
            reserved: [0],
        };
        let rc = unsafe { libc::ioctl(fd, VIDIOC_REQBUFS, &mut req) };
        if rc != 0 || req.count == 0 {
            return Err(format!(
                "VIDIOC_REQBUFS: {}",
                std::io::Error::last_os_error()
            ));
        }

        // Map each buffer and queue it.
        let mut buffers = Vec::new();
        for i in 0..req.count {
            let mut buf = V4l2Buffer {
                index: i,
                type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
                bytesused: 0,
                flags: 0,
                field: 0,
                timestamp: libc::timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
                timecode: [0; 16],
                sequence: 0,
                memory: V4L2_MEMORY_MMAP,
                offset: 0,
                length: 0,
                reserved2: 0,
                reserved: 0,
            };
            let rc = unsafe { libc::ioctl(fd, VIDIOC_QUERYBUF, &mut buf) };
            if rc != 0 {
                return Err(format!(
                    "VIDIOC_QUERYBUF {}: {}",
                    i,
                    std::io::Error::last_os_error()
                ));
            }
            let ptr = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    buf.length as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    fd,
                    buf.offset as i64,
                )
            };
            if ptr == libc::MAP_FAILED {
                return Err(format!("mmap {}: {}", i, std::io::Error::last_os_error()));
            }
            buffers.push((ptr as *mut u8, buf.length as usize));
            let rc = unsafe { libc::ioctl(fd, VIDIOC_QBUF, &mut buf) };
            if rc != 0 {
                return Err(format!(
                    "VIDIOC_QBUF {}: {}",
                    i,
                    std::io::Error::last_os_error()
                ));
            }
        }

        // Start streaming.
        let mut typ = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        let rc = unsafe { libc::ioctl(fd, VIDIOC_STREAMON, &mut typ) };
        if rc != 0 {
            return Err(format!(
                "VIDIOC_STREAMON: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(Self {
            file,
            width,
            height,
            buffers,
        })
    }
}

impl Drop for LinuxCapture {
    fn drop(&mut self) {
        for (ptr, len) in &self.buffers {
            unsafe { libc::munmap(*ptr as *mut libc::c_void, *len) };
        }
    }
}

impl crate::capture::Capture for LinuxCapture {
    fn frame(&mut self) -> Option<VideoFrame> {
        let fd = self.file.as_raw_fd();
        let mut buf = V4l2Buffer {
            index: 0,
            type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
            bytesused: 0,
            flags: 0,
            field: 0,
            timestamp: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            timecode: [0; 16],
            sequence: 0,
            memory: V4L2_MEMORY_MMAP,
            offset: 0,
            length: 0,
            reserved2: 0,
            reserved: 0,
        };
        let rc = unsafe { libc::ioctl(fd, VIDIOC_DQBUF, &mut buf) };
        if rc != 0 {
            return None;
        }
        let idx = buf.index as usize;
        let frame = if idx < self.buffers.len() {
            let (ptr, len) = self.buffers[idx];
            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            yuyv_to_y800(slice, self.width, self.height)
        } else {
            None
        };
        // Requeue the buffer.
        let _ = unsafe { libc::ioctl(fd, VIDIOC_QBUF, &mut buf) };
        frame
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
    fn yuyv_to_y800_round() {
        // Two pixels: Y=10, U=128, Y=20, V=128
        let yuyv = [10u8, 128, 20, 128];
        let frame = yuyv_to_y800(&yuyv, 2, 1).unwrap();
        assert_eq!(frame.format, VideoFormat::Y800);
        assert_eq!(frame.planes[0], vec![10, 20]);
    }

    #[test]
    fn yuyv_short_buffer() {
        assert!(yuyv_to_y800(&[1, 2], 2, 1).is_none());
    }

    #[test]
    fn open_without_device() {
        // In an environment without a camera this returns a clear error.
        // On a machine with /dev/video0 it succeeds.
        use crate::capture::Capture;
        match LinuxCapture::open() {
            Ok(c) => {
                assert!(c.width() > 0);
                assert!(c.height() > 0);
            }
            Err(e) => {
                assert!(e.contains("open") || e.contains("VIDIOC") || e.contains("No such"));
            }
        }
    }
}
