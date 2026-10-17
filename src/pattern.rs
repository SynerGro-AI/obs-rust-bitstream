//! Safe pure Pattern (checker fill core).
//!
//! Fills a Y800 frame with a checker of two values.
//! No unwraps.
//!
//! Differential: adjacent pixels differ.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternError {
    WrongFormat,
}

pub struct Pattern;

impl Pattern {
    /// size is the checker cell size in pixels.
    pub fn apply(frame: &mut VideoFrame, a: u8, b: u8, size: u8) -> Result<(), PatternError> {
        if frame.format != VideoFormat::Y800 {
            return Err(PatternError::WrongFormat);
        }
        let cell = size.max(1) as usize;
        let w = frame.width as usize;
        if let Some(plane) = frame.planes.get_mut(0) {
            for (i, p) in plane.iter_mut().enumerate() {
                let x = i % w;
                let y = i / w;
                let on = ((x / cell) + (y / cell)) % 2 == 0;
                *p = if on { a } else { b };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 2).unwrap();
        Pattern::apply(&mut f, 255, 0, 1).unwrap();
        assert_eq!(f.planes[0][0], 255);
        assert_eq!(f.planes[0][1], 0);
    }
}
