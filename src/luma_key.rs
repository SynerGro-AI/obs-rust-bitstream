//! Safe pure LumaKey (luminance key core).
//!
//! Marks pixels inside a luminance range as keyed.
//! Works on Y800. No unwraps.
//!
//! Differential: a mid pixel is keyed, a bright pixel is not.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LumaKeyError {
    WrongFormat,
}

pub struct LumaKey;

impl LumaKey {
    /// low/high define the keyed range. Keyed pixels become 0.
    pub fn apply(frame: &mut VideoFrame, low: u8, high: u8) -> Result<(), LumaKeyError> {
        if frame.format != VideoFormat::Y800 {
            return Err(LumaKeyError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                if *p >= low && *p <= high {
                    *p = 0;
                }
            }
        }
        Ok(())
    }

    /// Read low/high from live weights.
    pub fn from_weights(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
    ) -> Result<(), LumaKeyError> {
        let low = weights.get("key.luma.low").unwrap_or(0.0) as u8;
        let high = weights.get("key.luma.high").unwrap_or(255.0) as u8;
        Self::apply(frame, low, high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![50, 128, 200, 10];
        LumaKey::apply(&mut f, 100, 150).unwrap();
        assert_eq!(f.planes[0][0], 50);
        assert_eq!(f.planes[0][1], 0);
        assert_eq!(f.planes[0][2], 200);
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(120);
        let mut w = LiveWeights::new();
        w.set("key.luma.low", 100.0).unwrap();
        w.set("key.luma.high", 150.0).unwrap();
        LumaKey::from_weights(&mut f, &w).unwrap();
        assert_eq!(f.planes[0][0], 0);
    }
}
