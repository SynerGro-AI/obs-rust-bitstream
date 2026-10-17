//! Safe pure RateCompare (encode rate at two sizes).
//!
//! Encodes the same content at two sizes and
//! returns both fps values. No unwraps.
//!
//! Differential: both rates are positive.

use crate::h264_codec::{H264Codec, H264Error};
use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use crate::write_timing::WriteTiming;

pub struct RateCompare;

impl RateCompare {
    /// Encode one frame at two sizes. Returns (small_fps, large_fps).
    pub fn fps(small: u32, large: u32) -> Result<(f32, f32), H264Error> {
        let small_fps = Self::one(small)?;
        let large_fps = Self::one(large)?;
        Ok((small_fps, large_fps))
    }

    fn one(size: u32) -> Result<f32, H264Error> {
        let mut frame = VideoFrame::new(VideoFormat::Y800, size, size)
            .ok_or(H264Error::Encode)?;
        frame.planes[0].fill(40);
        let weights = LiveWeights::new();
        let mut enc = H264Codec::new(size, size, &weights)?;
        let timing = WriteTiming::start();
        let mut bytes = enc.encode(&frame)?;
        bytes.extend(enc.flush());
        let _ = bytes.len();
        Ok(timing.fps(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let (small, large) = RateCompare::fps(16, 32).unwrap();
        assert!(small > 0.0);
        assert!(large > 0.0);
    }
}
