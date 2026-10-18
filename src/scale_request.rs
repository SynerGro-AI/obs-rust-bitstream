//! Safe pure ScaleRequest (frame → scale info match).
//!
//! Checks whether a VideoFrame matches a VideoScaleInfo.
//! Returns the pixel delta if sizes differ. No unwraps.
//!
//! Differential: a larger target produces a larger delta.

use crate::video_frame::VideoFrame;
use crate::video_scale_info::VideoScaleInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaleMatch {
    pub matches: bool,
    pub src_pixels: u64,
    pub dst_pixels: u64,
}

impl ScaleMatch {
    /// True if the frame size and format match the scale info.
    pub fn from(frame: &VideoFrame, info: &VideoScaleInfo) -> Self {
        let src = frame.width as u64 * frame.height as u64;
        let dst = info.pixels();
        let matches = frame.width == info.width()
            && frame.height == info.height()
            && frame.format == info.format();
        Self {
            matches,
            src_pixels: src,
            dst_pixels: dst,
        }
    }

    /// Absolute pixel difference.
    pub fn delta(&self) -> u64 {
        self.src_pixels.abs_diff(self.dst_pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn match_same() {
        let f = VideoFrame::new(VideoFormat::Y800, 640, 480).unwrap();
        let i = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let m = ScaleMatch::from(&f, &i);
        assert!(m.matches);
        assert_eq!(m.delta(), 0);
    }

    #[test]
    fn differ() {
        let f = VideoFrame::new(VideoFormat::Y800, 640, 480).unwrap();
        let i = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let m = ScaleMatch::from(&f, &i);
        assert!(!m.matches);
        assert!(m.delta() > 0);
    }

    #[test]
    fn differential() {
        let f = VideoFrame::new(VideoFormat::Y800, 320, 240).unwrap();
        let a = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let b = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let ma = ScaleMatch::from(&f, &a);
        let mb = ScaleMatch::from(&f, &b);
        assert!(mb.delta() > ma.delta());
    }
}
