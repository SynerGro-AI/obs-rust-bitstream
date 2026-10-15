//! Safe pure FrameRenderer (color source → video frame).
//!
//! Fills a Y800 VideoFrame from a ColorSource. Zero size
//! rejected. No unwraps.
//!
//! Differential: a red source fills every pixel with the
//! luminance of red.

use crate::color_source::ColorSource;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    ZeroSize,
}

pub struct FrameRenderer;

impl FrameRenderer {
    /// Render a solid color into a Y800 frame. Luminance is
    /// the BT.601 approximation of the RGB color.
    pub fn solid(color: &ColorSource, width: u32, height: u32) -> Result<VideoFrame, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::ZeroSize);
        }
        let mut frame = VideoFrame::new(VideoFormat::Y800, width, height)
            .ok_or(RenderError::ZeroSize)?;
        let (r, g, b) = color.rgb();
        let y = ((77 * r as u32 + 150 * g as u32 + 29 * b as u32) / 256) as u8;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = y;
            }
        }
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let c = ColorSource::new(255, 0, 0);
        assert!(FrameRenderer::solid(&c, 0, 10).is_err());
    }

    #[test]
    fn differential() {
        let c = ColorSource::new(255, 0, 0);
        let f = FrameRenderer::solid(&c, 2, 2).unwrap();
        assert_eq!(f.width, 2);
        assert_eq!(f.height, 2);
        assert_eq!(f.format, VideoFormat::Y800);
        let y = f.planes[0][0];
        assert!(y > 50); // red is not black
        assert_eq!(f.planes[0].iter().all(|p| *p == y), true);
    }
}
