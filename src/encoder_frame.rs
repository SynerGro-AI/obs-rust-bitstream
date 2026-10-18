//! Safe pure EncoderFrame (OBS encoder_frame).
//!
//! Parsed from libobs/obs-encoder.h.
//! Holds plane data, linesize, frame count, and pts.
//! Built from a VideoFrame or from audio samples.
//! No unwraps.
//!
//! Differential: two frames with different pts differ.

use crate::video_frame::VideoFrame;

/// Maximum planes, matching OBS MAX_AV_PLANES (usually 8).
pub const MAX_AV_PLANES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncoderFrame {
    /// Plane data. Unused planes are empty.
    pub data: [Vec<u8>; MAX_AV_PLANES],
    /// Linesize of each plane.
    pub linesize: [u32; MAX_AV_PLANES],
    /// Number of audio frames. Zero for video.
    pub frames: u32,
    /// Presentation timestamp.
    pub pts: i64,
}

impl EncoderFrame {
    /// Empty frame with a pts.
    pub fn new(pts: i64) -> Self {
        Self {
            data: std::array::from_fn(|_| Vec::new()),
            linesize: [0; MAX_AV_PLANES],
            frames: 0,
            pts,
        }
    }

    /// Build from a VideoFrame. Uses plane 0.
    /// Returns None if the frame has no data.
    pub fn from_video(frame: &VideoFrame, pts: i64) -> Option<Self> {
        if frame.data.is_empty() {
            return None;
        }
        let mut ef = Self::new(pts);
        ef.data[0] = frame.data.clone();
        ef.linesize[0] = frame.width;
        Some(ef)
    }

    /// Build from audio samples. Uses plane 0.
    /// frames is the sample count. Returns None if empty.
    pub fn from_audio(samples: &[f32], pts: i64) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let mut bytes = Vec::with_capacity(samples.len() * 4);
        for s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        let mut ef = Self::new(pts);
        ef.data[0] = bytes;
        ef.linesize[0] = 4;
        ef.frames = samples.len() as u32;
        Some(ef)
    }

    /// True if plane 0 has data.
    pub fn has_data(&self) -> bool {
        !self.data[0].is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    #[test]
    fn from_video() {
        let mut vf = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        vf.data[0] = 128;
        let ef = EncoderFrame::from_video(&vf, 42).unwrap();
        assert_eq!(ef.pts, 42);
        assert_eq!(ef.linesize[0], 16);
        assert!(ef.has_data());
        assert_eq!(ef.frames, 0);
    }

    #[test]
    fn from_audio() {
        let samples = [0.0_f32, 0.5, -0.5];
        let ef = EncoderFrame::from_audio(&samples, 7).unwrap();
        assert_eq!(ef.pts, 7);
        assert_eq!(ef.frames, 3);
        assert_eq!(ef.linesize[0], 4);
        assert_eq!(ef.data[0].len(), 12);
    }

    #[test]
    fn empty_is_none() {
        let vf = VideoFrame::new(VideoFormat::Y800, 0, 0);
        assert!(vf.is_err());
        assert!(EncoderFrame::from_audio(&[], 0).is_none());
    }

    #[test]
    fn differential() {
        let mut vf = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        vf.data[0] = 1;
        let a = EncoderFrame::from_video(&vf, 0).unwrap();
        let b = EncoderFrame::from_video(&vf, 1).unwrap();
        assert_ne!(a.pts, b.pts);
    }
}
