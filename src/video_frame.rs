//! Safe pure video frame (OBS video-frame.h core).
//!
//! The C version allocates plane buffers based on format and size.
//! The safe path owns a Vec of planes and computes linesizes without
//! raw pointers. Zero dimensions return None. No unwraps.
//!
//! Differential: plane count and sizes match the format rules.

#[cfg(test)]
use crate::diff::assert_equal;
use crate::video_fourcc::VideoFormat;

#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub planes: Vec<Vec<u8>>,
    pub linesizes: Vec<u32>,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
}

impl VideoFrame {
    pub fn new(format: VideoFormat, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let (planes, linesizes) = match format {
            VideoFormat::Y800 => {
                let ls = width;
                (vec![vec![0u8; (ls as usize) * (height as usize)]], vec![ls])
            }
            VideoFormat::Uyvy | VideoFormat::Yuy2 | VideoFormat::Yvyu => {
                let ls = width * 2;
                (vec![vec![0u8; (ls as usize) * (height as usize)]], vec![ls])
            }
            VideoFormat::None => return None,
        };
        Some(Self {
            planes,
            linesizes,
            width,
            height,
            format,
        })
    }

    pub fn plane_count(&self) -> usize {
        self.planes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn y800() {
        let f = VideoFrame::new(VideoFormat::Y800, 4, 2).unwrap();
        assert_eq!(f.plane_count(), 1);
        assert_eq!(f.linesizes[0], 4);
        assert_eq!(f.planes[0].len(), 8);
    }

    #[test]
    fn yuy2() {
        let f = VideoFrame::new(VideoFormat::Yuy2, 4, 2).unwrap();
        assert_eq!(f.linesizes[0], 8);
        assert_eq!(f.planes[0].len(), 16);
    }

    #[test]
    fn zero_safe() {
        assert!(VideoFrame::new(VideoFormat::Y800, 0, 10).is_none());
        assert!(VideoFrame::new(VideoFormat::None, 10, 10).is_none());
    }

    #[test]
    fn differential() {
        let f = VideoFrame::new(VideoFormat::Uyvy, 2, 2).unwrap();
        assert_equal(f.plane_count(), 1, "plane count");
        assert_equal(f.planes[0].len(), 8, "uyvy size");
    }
}
