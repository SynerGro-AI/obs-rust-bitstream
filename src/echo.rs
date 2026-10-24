//! Safe pure Echo (frame blend core).
//!
//! Blends the current frame with a previous frame.
//! Works on Y800. No unwraps.
//!
//! Differential: a previous bright pixel lifts the current.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EchoError {
    WrongFormat,
    SizeMismatch,
}

pub struct Echo;

impl Echo {
    /// amount is 0..=255. 255 is full previous.
    pub fn apply(
        current: &mut VideoFrame,
        previous: &VideoFrame,
        amount: u8,
    ) -> Result<(), EchoError> {
        if current.format != VideoFormat::Y800 || previous.format != VideoFormat::Y800 {
            return Err(EchoError::WrongFormat);
        }
        if current.width != previous.width || current.height != previous.height {
            return Err(EchoError::SizeMismatch);
        }
        if let (Some(dst), Some(prev)) = (current.planes.get_mut(0), previous.planes.first()) {
            for (d, p) in dst.iter_mut().zip(prev.iter()) {
                let v = (*d as u16 * (255 - amount as u16) + *p as u16 * amount as u16) / 255;
                *d = v as u8;
            }
        }
        Ok(())
    }

    /// Read amount from live weights.
    pub fn from_weights(
        current: &mut VideoFrame,
        previous: &VideoFrame,
        weights: &LiveWeights,
    ) -> Result<(), EchoError> {
        let v = weights.get("time.echo").unwrap_or(0.0);
        Self::apply(current, previous, v.clamp(0.0, 255.0) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut cur = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        cur.planes[0].fill(0);
        let mut prev = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        prev.planes[0].fill(200);
        Echo::apply(&mut cur, &prev, 128).unwrap();
        assert!(cur.planes[0][0] > 0 && cur.planes[0][0] < 200);
    }
}
