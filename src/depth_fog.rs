//! Safe pure DepthFog (depth fade core).
//!
//! Darkens pixels by a depth value. Works on Y800.
//! No unwraps.
//!
//! Differential: a far pixel is darker than a near pixel.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DepthError {
    WrongFormat,
    SizeMismatch,
}

pub struct DepthFog;

impl DepthFog {
    /// depth is a Y800 frame where higher means farther.
    /// density is 0..=255.
    pub fn apply(
        frame: &mut VideoFrame,
        depth: &VideoFrame,
        density: u8,
    ) -> Result<(), DepthError> {
        if frame.format != VideoFormat::Y800 || depth.format != VideoFormat::Y800 {
            return Err(DepthError::WrongFormat);
        }
        if frame.width != depth.width || frame.height != depth.height {
            return Err(DepthError::SizeMismatch);
        }
        if let (Some(dst), Some(d)) = (frame.planes.get_mut(0), depth.planes.first()) {
            for (p, dv) in dst.iter_mut().zip(d.iter()) {
                let fade = (*dv as u16 * density as u16) / 255;
                let v = (*p as u16).saturating_sub(fade);
                *p = v as u8;
            }
        }
        Ok(())
    }

    /// Read density from live weights.
    pub fn from_weights(
        frame: &mut VideoFrame,
        depth: &VideoFrame,
        weights: &LiveWeights,
    ) -> Result<(), DepthError> {
        let v = weights.get("depth.density").unwrap_or(0.0);
        Self::apply(frame, depth, v.clamp(0.0, 255.0) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(200);
        let mut d = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        d.planes[0] = vec![0, 255, 0, 255];
        DepthFog::apply(&mut f, &d, 128).unwrap();
        assert_eq!(f.planes[0][0], 200);
        assert!(f.planes[0][1] < 200);
    }
}
