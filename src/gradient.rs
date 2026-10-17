//! Safe pure Gradient (linear ramp core).
//!
//! Fills a Y800 frame with a left-to-right ramp from
//! start to end. No unwraps.
//!
//! Differential: the left edge is start, the right is end.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GradientError {
    WrongFormat,
}

pub struct Gradient;

impl Gradient {
    pub fn apply(frame: &mut VideoFrame, start: u8, end: u8) -> Result<(), GradientError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GradientError::WrongFormat);
        }
        let w = frame.width as usize;
        if let Some(plane) = frame.planes.get_mut(0) {
            for (i, p) in plane.iter_mut().enumerate() {
                let x = i % w;
                let t = if w <= 1 { 0 } else { (x * 255) / (w - 1) };
                let v = start as u16 + ((end as i16 - start as i16) as u16 * t as u16) / 255;
                *p = v.min(255) as u8;
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
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 1).unwrap();
        Gradient::apply(&mut f, 0, 255).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][3], 255);
    }
}
