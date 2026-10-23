//! Safe pure QualityCompare (size at two QP values).
//!
//! Encodes the same frame at two QP settings and
//! returns both sizes. No unwraps.
//!
//! Differential: higher QP is smaller or equal.

use crate::h264_codec::{H264Codec, H264Error};
use crate::live_weights::LiveWeights;
use crate::video_frame::VideoFrame;

pub struct QualityCompare;

impl QualityCompare {
    /// Return (low_qp_size, high_qp_size).
    pub fn sizes(
        frame: &VideoFrame,
        low_qp: f32,
        high_qp: f32,
    ) -> Result<(usize, usize), H264Error> {
        let mut w_low = LiveWeights::new();
        w_low
            .set("h264.qp", low_qp)
            .map_err(|_| H264Error::Encode)?;
        let mut w_high = LiveWeights::new();
        w_high
            .set("h264.qp", high_qp)
            .map_err(|_| H264Error::Encode)?;
        let mut enc_low = H264Codec::new(frame.width, frame.height, &w_low)?;
        let mut enc_high = H264Codec::new(frame.width, frame.height, &w_high)?;
        let low = enc_low.encoded_size(frame)?;
        let high = enc_high.encoded_size(frame)?;
        Ok((low, high))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        f.planes[0].fill(40);
        let (low, high) = QualityCompare::sizes(&f, 20.0, 40.0).unwrap();
        assert!(low > 0);
        assert!(high > 0);
        // higher QP should not be larger
        assert!(high <= low);
    }
}
