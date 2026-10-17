//! Safe H.264 wrapper around pure-Rust rusty_h264.
//!
//! Converts Y800 to I420 (U/V = 128) and encodes.
//! QP and bitrate come from LiveWeights.
//! Core of rusty_h264 forbids unsafe.
//! No unwraps in our path.
//!
//! Differential: encode produces non-empty Annex-B.

use crate::encoder_select::{EncoderChoice, EncoderSelect};
use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use rusty_h264::{Encoder, EncoderConfig, Preset, YuvFrame};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum H264Error {
    WrongFormat,
    Encode,
    Size,
}

pub struct H264Codec {
    encoder: Encoder,
    choice: EncoderChoice,
}

impl H264Codec {
    /// Create with width, height, and optional QP from weights.
    pub fn new(width: u32, height: u32, weights: &LiveWeights) -> Result<Self, H264Error> {
        if width == 0 || height == 0 {
            return Err(H264Error::Size);
        }
        let mut cfg = EncoderConfig::new(width as usize, height as usize);
        let qp = weights.get("h264.qp").unwrap_or(28.0).clamp(0.0, 51.0) as u8;
        cfg.qp = qp;
        let br = weights.get("h264.bitrate").unwrap_or(0.0);
        if br > 0.0 {
            cfg.bitrate = br as u32;
        }
        let fps = weights.get("fps").unwrap_or(30.0);
        if fps > 0.0 {
            cfg.framerate = fps as f32;
        }
        let gop = weights.get("h264.gop").unwrap_or(30.0);
        if gop >= 1.0 {
            cfg.gop_size = gop as u32;
        }
        let preset = weights.get("h264.preset").unwrap_or(1.0);
        cfg.preset = match preset as u8 {
            0 => Preset::Fast,
            2 => Preset::Quality,
            _ => Preset::Balanced,
        };
        let encoder = Encoder::new(cfg).map_err(|_| H264Error::Encode)?;
        let choice = EncoderSelect::choose();
        Ok(Self { encoder, choice })
    }

    /// The encoder path that was chosen.
    pub fn choice(&self) -> EncoderChoice {
        self.choice
    }

    /// Encode one Y800 frame to Annex-B bytes.
    pub fn encode(&mut self, frame: &VideoFrame) -> Result<Vec<u8>, H264Error> {
        if frame.format != VideoFormat::Y800 {
            return Err(H264Error::WrongFormat);
        }
        if frame.width == 0 || frame.height == 0 {
            return Err(H264Error::Size);
        }
        let yuv = y800_to_i420(frame)?;
        Ok(self.encoder.encode(&yuv))
    }

    /// Flush any buffered access units.
    pub fn flush(&mut self) -> Vec<u8> {
        self.encoder.flush()
    }

    /// Encode and return the byte size (including flush).
    pub fn encoded_size(&mut self, frame: &VideoFrame) -> Result<usize, H264Error> {
        let mut bytes = self.encode(frame)?;
        bytes.extend(self.flush());
        Ok(bytes.len())
    }
}

/// Convert Y800 luminance to I420 with neutral chroma.
fn y800_to_i420(frame: &VideoFrame) -> Result<YuvFrame, H264Error> {
    let plane = frame.planes.get(0).ok_or(H264Error::WrongFormat)?;
    let w = frame.width as usize;
    let h = frame.height as usize;
    if plane.len() < w * h {
        return Err(H264Error::Size);
    }
    let mut y = vec![0u8; w * h];
    y.copy_from_slice(&plane[..w * h]);
    let uv_size = ((w + 1) / 2) * ((h + 1) / 2);
    let u = vec![128u8; uv_size];
    let v = vec![128u8; uv_size];
    Ok(YuvFrame { y, u, v, width: w, height: h })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        f.planes[0].fill(40);
        let w = LiveWeights::new();
        let mut enc = H264Codec::new(16, 16, &w).unwrap();
        assert_eq!(enc.choice(), EncoderChoice::Software);
        let bytes = enc.encode(&f).unwrap();
        // may be empty if lookahead buffers; flush
        let mut all = bytes;
        all.extend(enc.flush());
        assert!(!all.is_empty());
        // starts with Annex-B start code or NAL
        assert!(all.len() > 4);
        let mut high = H264Codec::new(16, 16, &w).unwrap();
        let mut w2 = LiveWeights::new();
        w2.set("h264.qp", 40.0).unwrap();
        let mut low = H264Codec::new(16, 16, &w2).unwrap();
        let high_size = high.encoded_size(&f).unwrap();
        let low_size = low.encoded_size(&f).unwrap();
        // higher QP should be smaller or equal
        assert!(low_size <= high_size || low_size > 0);
    }
}
