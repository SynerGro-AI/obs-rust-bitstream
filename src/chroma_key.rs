//! Safe pure ChromaKey (single-channel key core).
//!
//! On Y800 a chroma key treats a narrow band as the
//! keyed color. Keyed pixels become 0. No unwraps.
//!
//! Differential: pixels near the key are removed.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChromaError {
    WrongFormat,
}

pub struct ChromaKey;

impl ChromaKey {
    /// key is the target value. tolerance is the band width.
    pub fn apply(frame: &mut VideoFrame, key: u8, tolerance: u8) -> Result<(), ChromaError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ChromaError::WrongFormat);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                if p.abs_diff(key) <= tolerance {
                    *p = 0;
                }
            }
        }
        Ok(())
    }

    /// Read key and tolerance from live weights.
    pub fn from_weights(frame: &mut VideoFrame, weights: &LiveWeights) -> Result<(), ChromaError> {
        let key = weights.get("key.chroma").unwrap_or(0.0) as u8;
        let tol = weights.get("key.chroma.tol").unwrap_or(10.0) as u8;
        Self::apply(frame, key, tol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![100, 108, 200, 90];
        ChromaKey::apply(&mut f, 100, 10).unwrap();
        assert_eq!(f.planes[0][0], 0);
        assert_eq!(f.planes[0][1], 0);
        assert_eq!(f.planes[0][2], 200);
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(50);
        let mut w = LiveWeights::new();
        w.set("key.chroma", 50.0).unwrap();
        w.set("key.chroma.tol", 5.0).unwrap();
        ChromaKey::from_weights(&mut f, &w).unwrap();
        assert_eq!(f.planes[0][0], 0);
    }
}
