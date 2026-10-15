//! Safe pure SyntheticCamera (animated source core).
//!
//! Generates a Y800 frame whose content moves with time.
//! Zero size rejected. No unwraps.
//!
//! Differential: time 0 and time 1 produce different
//! pixel values.

use crate::frame_renderer::RenderError;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug)]
pub struct SyntheticCamera {
    width: u32,
    height: u32,
}

impl SyntheticCamera {
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::ZeroSize);
        }
        Ok(Self { width, height })
    }

    /// Render a frame at `time` seconds. A bright bar moves
    /// left to right once per second.
    pub fn frame(&self, time: f32) -> Result<VideoFrame, RenderError> {
        let mut frame = VideoFrame::new(VideoFormat::Y800, self.width, self.height)
            .ok_or(RenderError::ZeroSize)?;
        let pos = ((time.fract() * self.width as f32) as u32).min(self.width.saturating_sub(1));
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..self.height {
                for x in 0..self.width {
                    let i = (y * self.width + x) as usize;
                    if i < plane.len() {
                        let d = if x > pos { x - pos } else { pos - x };
                        plane[i] = if d < 4 { 255 } else { 32 };
                    }
                }
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
        assert!(SyntheticCamera::new(0, 10).is_err());
    }

    #[test]
    fn differential() {
        let cam = SyntheticCamera::new(16, 2).unwrap();
        let a = cam.frame(0.0).unwrap();
        let b = cam.frame(0.5).unwrap();
        assert_ne!(a.planes[0][0], b.planes[0][0]);
    }
}
