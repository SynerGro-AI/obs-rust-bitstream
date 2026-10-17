//! Safe pure MotionBlur (directional smear core).
//!
//! Averages each pixel with the pixel `distance` steps away.
//! Works on Y800. No unwraps.
//!
//! Differential: a bright pixel smears right.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MotionError {
    WrongFormat,
}

pub struct MotionBlur;

impl MotionBlur {
    /// distance is pixels to the right.
    pub fn apply(frame: &mut VideoFrame, distance: u8) -> Result<(), MotionError> {
        if frame.format != VideoFormat::Y800 {
            return Err(MotionError::WrongFormat);
        }
        let w = frame.width as usize;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            let d = distance.max(1) as usize;
            for i in 0..src.len() {
                let x = i % w;
                let sample = if x >= d { src[i - d] } else { src[i] };
                dst[i] = ((src[i] as u16 + sample as u16) / 2) as u8;
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
        f.planes[0] = vec![0, 0, 255, 0];
        MotionBlur::apply(&mut f, 1).unwrap();
        assert!(f.planes[0][2] < 255);
        assert!(f.planes[0][1] > 0 || f.planes[0][3] == 0);
    }
}
