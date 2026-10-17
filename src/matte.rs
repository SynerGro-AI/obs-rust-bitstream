//! Safe pure Matte (track and set matte core).
//!
//! Track matte uses one frame as the alpha for another.
//! Set matte fills a frame with a constant. Works on Y800.
//! No unwraps.
//!
//! Differential: a bright matte keeps the pixel, a dark
//! matte clears it.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatteError {
    WrongFormat,
    SizeMismatch,
}

pub struct Matte;

impl Matte {
    /// Keep pixels in `frame` where `matte` is above threshold.
    pub fn track(
        frame: &mut VideoFrame,
        matte: &VideoFrame,
        threshold: u8,
    ) -> Result<(), MatteError> {
        if frame.format != VideoFormat::Y800 || matte.format != VideoFormat::Y800 {
            return Err(MatteError::WrongFormat);
        }
        if frame.width != matte.width || frame.height != matte.height {
            return Err(MatteError::SizeMismatch);
        }
        if let (Some(dst), Some(m)) = (frame.planes.get_mut(0), matte.planes.get(0)) {
            for (d, mv) in dst.iter_mut().zip(m.iter()) {
                if *mv < threshold {
                    *d = 0;
                }
            }
        }
        Ok(())
    }

    /// Fill the frame with a constant matte value.
    pub fn set(frame: &mut VideoFrame, value: u8) -> Result<(), MatteError> {
        if frame.format != VideoFormat::Y800 {
            return Err(MatteError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            plane.fill(value);
        }
        Ok(())
    }

    /// Read threshold from live weights.
    pub fn track_from_weights(
        frame: &mut VideoFrame,
        matte: &VideoFrame,
        weights: &LiveWeights,
    ) -> Result<(), MatteError> {
        let t = weights.get("matte.threshold").unwrap_or(128.0) as u8;
        Self::track(frame, matte, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        let m = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        assert!(Matte::track(&mut f, &m, 128).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(200);
        let mut m = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        m.planes[0] = vec![255, 0, 255, 0];
        Matte::track(&mut f, &m, 128).unwrap();
        assert_eq!(f.planes[0][0], 200);
        assert_eq!(f.planes[0][1], 0);
        Matte::set(&mut f, 64).unwrap();
        assert_eq!(f.planes[0][0], 64);
    }
}
