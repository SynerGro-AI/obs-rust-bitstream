//! Safe pure Codec (Y800 encode/decode core).
//!
//! Encodes a Y800 frame to bytes and decodes them
//! back. Uses a simple run-length scheme. No unwraps.
//!
//! Differential: encode + decode round-trips.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    WrongFormat,
    BadData,
}

pub struct Codec;

impl Codec {
    /// Encode a Y800 frame to bytes.
    /// Format: width(4) + height(4) + run-length pairs.
    pub fn encode(frame: &VideoFrame) -> Result<Vec<u8>, CodecError> {
        if frame.format != VideoFormat::Y800 {
            return Err(CodecError::WrongFormat);
        }
        let plane = frame.planes.get(0).ok_or(CodecError::WrongFormat)?;
        let mut out = Vec::new();
        out.extend_from_slice(&frame.width.to_le_bytes());
        out.extend_from_slice(&frame.height.to_le_bytes());
        if plane.is_empty() {
            return Ok(out);
        }
        let mut run_val = plane[0];
        let mut run_len: u8 = 1;
        for &p in &plane[1..] {
            if p == run_val && run_len < 255 {
                run_len += 1;
            } else {
                out.push(run_val);
                out.push(run_len);
                run_val = p;
                run_len = 1;
            }
        }
        out.push(run_val);
        out.push(run_len);
        Ok(out)
    }

    /// Decode bytes back to a Y800 frame.
    pub fn decode(data: &[u8]) -> Result<VideoFrame, CodecError> {
        if data.len() < 8 {
            return Err(CodecError::BadData);
        }
        let width = u32::from_le_bytes(data[0..4].try_into().map_err(|_| CodecError::BadData)?);
        let height = u32::from_le_bytes(data[4..8].try_into().map_err(|_| CodecError::BadData)?);
        let mut frame = VideoFrame::new(VideoFormat::Y800, width, height)
            .ok_or(CodecError::BadData)?;
        let expected = (width as usize) * (height as usize);
        let mut pixels = Vec::with_capacity(expected);
        let mut i = 8;
        while i + 1 < data.len() && pixels.len() < expected {
            let val = data[i];
            let len = data[i + 1] as usize;
            for _ in 0..len {
                if pixels.len() < expected {
                    pixels.push(val);
                }
            }
            i += 2;
        }
        if pixels.len() != expected {
            return Err(CodecError::BadData);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            plane.copy_from_slice(&pixels);
        }
        Ok(frame)
    }

    /// Encode with a quality weight. Quality 0 skips encode.
    pub fn encode_weighted(
        frame: &VideoFrame,
        weights: &LiveWeights,
    ) -> Result<Vec<u8>, CodecError> {
        let q = weights.get("codec.quality").unwrap_or(1.0);
        if q <= 0.0 {
            return Ok(Vec::new());
        }
        Self::encode(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad() {
        assert!(Codec::decode(&[]).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 2).unwrap();
        f.planes[0] = vec![10, 10, 10, 20, 20, 30, 30, 30];
        let bytes = Codec::encode(&f).unwrap();
        let back = Codec::decode(&bytes).unwrap();
        assert_eq!(back.planes[0], f.planes[0]);
        let mut w = LiveWeights::new();
        w.set("codec.quality", 0.0).unwrap();
        let empty = Codec::encode_weighted(&f, &w).unwrap();
        assert!(empty.is_empty());
    }
}
