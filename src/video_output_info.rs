//! Safe pure VideoOutputInfo (OBS video_output_info).
//!
//! Parsed from libobs/media-io/video-io.h.
//! Holds name, format, fps, size, cache, colorspace, and range.
//! Empty name or zero size is rejected. No unwraps.
//!
//! Differential: a larger size produces more pixels.

use crate::video_fourcc::VideoFormat;
use crate::video_scale_info::{VideoColorspace, VideoRange};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputInfoError {
    EmptyName,
    ZeroSize,
    ZeroFps,
}

#[derive(Clone, Debug)]
pub struct VideoOutputInfo {
    name: String,
    format: VideoFormat,
    fps_num: u32,
    fps_den: u32,
    width: u32,
    height: u32,
    cache_size: usize,
    colorspace: VideoColorspace,
    range: VideoRange,
}

impl VideoOutputInfo {
    /// Create a video output info. Empty name, zero size, or zero fps is rejected.
    pub fn new(
        name: &str,
        format: VideoFormat,
        fps_num: u32,
        fps_den: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, OutputInfoError> {
        if name.is_empty() {
            return Err(OutputInfoError::EmptyName);
        }
        if width == 0 || height == 0 {
            return Err(OutputInfoError::ZeroSize);
        }
        if fps_num == 0 || fps_den == 0 {
            return Err(OutputInfoError::ZeroFps);
        }
        Ok(Self {
            name: name.to_string(),
            format,
            fps_num,
            fps_den,
            width,
            height,
            cache_size: 0,
            colorspace: VideoColorspace::Default,
            range: VideoRange::Default,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn format(&self) -> VideoFormat {
        self.format
    }

    pub fn fps_num(&self) -> u32 {
        self.fps_num
    }

    pub fn fps_den(&self) -> u32 {
        self.fps_den
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cache_size(&self) -> usize {
        self.cache_size
    }

    pub fn set_cache_size(&mut self, n: usize) {
        self.cache_size = n;
    }

    pub fn colorspace(&self) -> VideoColorspace {
        self.colorspace
    }

    pub fn set_colorspace(&mut self, cs: VideoColorspace) {
        self.colorspace = cs;
    }

    pub fn range(&self) -> VideoRange {
        self.range
    }

    pub fn set_range(&mut self, range: VideoRange) {
        self.range = range;
    }

    /// Frame rate as a float.
    pub fn fps(&self) -> f64 {
        self.fps_num as f64 / self.fps_den as f64
    }

    /// Pixel count.
    pub fn pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let v = VideoOutputInfo::new("main", VideoFormat::Y800, 30, 1, 1920, 1080).unwrap();
        assert_eq!(v.name(), "main");
        assert_eq!(v.fps(), 30.0);
        assert_eq!(v.pixels(), 1920 * 1080);
    }

    #[test]
    fn rejects() {
        assert!(VideoOutputInfo::new("", VideoFormat::Y800, 30, 1, 100, 100).is_err());
        assert!(VideoOutputInfo::new("main", VideoFormat::Y800, 0, 1, 100, 100).is_err());
        assert!(VideoOutputInfo::new("main", VideoFormat::Y800, 30, 1, 0, 100).is_err());
    }

    #[test]
    fn setters() {
        let mut v = VideoOutputInfo::new("main", VideoFormat::Y800, 60, 1, 640, 480).unwrap();
        v.set_cache_size(4);
        v.set_colorspace(VideoColorspace::Bt709);
        v.set_range(VideoRange::Full);
        assert_eq!(v.cache_size(), 4);
        assert_eq!(v.colorspace(), VideoColorspace::Bt709);
        assert_eq!(v.range(), VideoRange::Full);
    }

    #[test]
    fn differential() {
        let a = VideoOutputInfo::new("a", VideoFormat::Y800, 30, 1, 640, 480).unwrap();
        let b = VideoOutputInfo::new("b", VideoFormat::Y800, 30, 1, 1920, 1080).unwrap();
        assert!(b.pixels() > a.pixels());
    }
}
