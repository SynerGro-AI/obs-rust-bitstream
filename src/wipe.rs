//! Safe pure Wipe (left-to-right reveal core).
//!
//! Reveals the destination from the left by progress.
//! Works on Y800. No unwraps.
//!
//! Differential: progress 0.5 shows the left half of b.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WipeError {
    WrongFormat,
    SizeMismatch,
}

pub struct Wipe;

impl Wipe {
    /// progress is 0.0..=1.0.
    pub fn apply(
        a: &VideoFrame,
        b: &VideoFrame,
        progress: f32,
    ) -> Result<VideoFrame, WipeError> {
        if a.format != VideoFormat::Y800 || b.format != VideoFormat::Y800 {
            return Err(WipeError::WrongFormat);
        }
        if a.width != b.width || a.height != b.height {
            return Err(WipeError::SizeMismatch);
        }
        let mut out = VideoFrame::new(VideoFormat::Y800, a.width, a.height)
            .ok_or(WipeError::SizeMismatch)?;
        let cut = (progress.clamp(0.0, 1.0) * a.width as f32) as u32;
        if let (Some(pa), Some(pb), Some(dst)) =
            (a.planes.get(0), b.planes.get(0), out.planes.get_mut(0))
        {
            for y in 0..a.height {
                for x in 0..a.width {
                    let i = (y * a.width + x) as usize;
                    dst[i] = if x < cut { pb[i] } else { pa[i] };
                }
            }
        }
        Ok(out)
    }

    /// Read progress from live weights.
    pub fn from_weights(
        a: &VideoFrame,
        b: &VideoFrame,
        weights: &LiveWeights,
    ) -> Result<VideoFrame, WipeError> {
        let p = weights.get("transition.wipe").unwrap_or(0.0);
        Self::apply(a, b, p.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut a = VideoFrame::new(VideoFormat::Y800, 4, 1).unwrap();
        a.planes[0].fill(0);
        let mut b = VideoFrame::new(VideoFormat::Y800, 4, 1).unwrap();
        b.planes[0].fill(255);
        let out = Wipe::apply(&a, &b, 0.5).unwrap();
        assert_eq!(out.planes[0][0], 255);
        assert_eq!(out.planes[0][3], 0);
    }
}
