//! Safe pure Crossfade (blend two sequences core).
//!
//! Blends frame A into frame B by progress. Works on
//! Y800. No unwraps.
//!
//! Differential: progress 0 is A, progress 1 is B.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrossError {
    WrongFormat,
    SizeMismatch,
}

pub struct Crossfade;

impl Crossfade {
    /// progress is 0.0..=1.0. 0 is all a, 1 is all b.
    pub fn blend(a: &VideoFrame, b: &VideoFrame, progress: f32) -> Result<VideoFrame, CrossError> {
        if a.format != VideoFormat::Y800 || b.format != VideoFormat::Y800 {
            return Err(CrossError::WrongFormat);
        }
        if a.width != b.width || a.height != b.height {
            return Err(CrossError::SizeMismatch);
        }
        let mut out = VideoFrame::new(VideoFormat::Y800, a.width, a.height)
            .ok_or(CrossError::SizeMismatch)?;
        let t = progress.clamp(0.0, 1.0);
        if let (Some(pa), Some(pb), Some(dst)) =
            (a.planes.first(), b.planes.first(), out.planes.get_mut(0))
        {
            for (d, (va, vb)) in dst.iter_mut().zip(pa.iter().zip(pb.iter())) {
                let v = *va as f32 * (1.0 - t) + *vb as f32 * t;
                *d = v as u8;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut a = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        a.planes[0].fill(0);
        let mut b = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        b.planes[0].fill(100);
        let mid = Crossfade::blend(&a, &b, 0.5).unwrap();
        assert_eq!(mid.planes[0][0], 50);
        let start = Crossfade::blend(&a, &b, 0.0).unwrap();
        assert_eq!(start.planes[0][0], 0);
    }
}
