//! Safe pure Grain (fine noise core).
//!
//! Adds small position-based variation. Works on Y800.
//! No unwraps.
//!
//! Differential: amount 30 changes a flat frame.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrainError {
    WrongFormat,
}

pub struct Grain;

impl Grain {
    /// amount is 0..=255. Uses a fine position hash.
    pub fn apply(frame: &mut VideoFrame, amount: u8) -> Result<(), GrainError> {
        if frame.format != VideoFormat::Y800 {
            return Err(GrainError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for (i, p) in plane.iter_mut().enumerate() {
                let n = ((i as u32).wrapping_mul(1664525).wrapping_add(1013904223) >> 24) as u8;
                let delta = (n as i16 - 128) * amount as i16 / 512;
                let v = (*p as i16 + delta).clamp(0, 255);
                *p = v as u8;
            }
        }
        Ok(())
    }

    /// Read amount from live weights.
    pub fn from_weights(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), GrainError> {
        let v = weights.get(name).unwrap_or(0.0);
        Self::apply(frame, v.clamp(0.0, 255.0) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(128);
        Grain::apply(&mut f, 80).unwrap();
        assert!(f.planes[0].iter().any(|p| *p != 128));
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(100);
        let mut w = LiveWeights::new();
        w.set("grain.amount", 60.0).unwrap();
        Grain::from_weights(&mut f, &w, "grain.amount").unwrap();
        assert!(f.planes[0].iter().any(|p| *p != 100));
    }
}
