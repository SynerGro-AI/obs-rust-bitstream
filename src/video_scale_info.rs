//! Safe pure VideoScaleInfo (OBS video_scale_info).
//!
//! Parsed from libobs/media-io/video-io.h.
//! Holds format, size, range, and colorspace for a
//! conversion request. Zero size is rejected. No unwraps.
//!
//! Differential: two infos with different widths differ.

use crate::video_fourcc::VideoFormat;

/// Matching OBS video_range_type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoRange {
    Default,
    Partial,
    Full,
}

/// Matching OBS video_colorspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoColorspace {
    Default,
    Bt601,
    Bt709,
    Srgb,
}

/// Matching OBS video_scale_type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoScaleType {
    Default,
    Point,
    FastBilinear,
    Bilinear,
    Bicubic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScaleError {
    ZeroSize,
}

#[derive(Clone, Debug)]
pub struct VideoScaleInfo {
    format: VideoFormat,
    width: u32,
    height: u32,
    range: VideoRange,
    colorspace: VideoColorspace,
    scale: VideoScaleType,
}

impl VideoScaleInfo {
    /// Create a scale request. Zero width or height is rejected.
    pub fn new(format: VideoFormat, width: u32, height: u32) -> Result<Self, ScaleError> {
        if width == 0 || height == 0 {
            return Err(ScaleError::ZeroSize);
        }
        Ok(Self {
            format,
            width,
            height,
            range: VideoRange::Default,
            colorspace: VideoColorspace::Default,
            scale: VideoScaleType::Default,
        })
    }

    pub fn format(&self) -> VideoFormat {
        self.format
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn range(&self) -> VideoRange {
        self.range
    }

    pub fn set_range(&mut self, range: VideoRange) {
        self.range = range;
    }

    pub fn colorspace(&self) -> VideoColorspace {
        self.colorspace
    }

    pub fn set_colorspace(&mut self, cs: VideoColorspace) {
        self.colorspace = cs;
    }

    pub fn scale(&self) -> VideoScaleType {
        self.scale
    }

    pub fn set_scale(&mut self, scale: VideoScaleType) {
        self.scale = scale;
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
        let s = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        assert_eq!(s.width(), 1920);
        assert_eq!(s.pixels(), 1920 * 1080);
        assert_eq!(s.range(), VideoRange::Default);
    }

    #[test]
    fn zero_rejected() {
        assert!(VideoScaleInfo::new(VideoFormat::Y800, 0, 1080).is_err());
        assert!(VideoScaleInfo::new(VideoFormat::Y800, 1920, 0).is_err());
    }

    #[test]
    fn setters() {
        let mut s = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        s.set_range(VideoRange::Full);
        s.set_colorspace(VideoColorspace::Bt709);
        s.set_scale(VideoScaleType::Bicubic);
        assert_eq!(s.range(), VideoRange::Full);
        assert_eq!(s.colorspace(), VideoColorspace::Bt709);
        assert_eq!(s.scale(), VideoScaleType::Bicubic);
    }

    #[test]
    fn differential() {
        let a = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let b = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        assert!(b.pixels() > a.pixels());
    }
}
