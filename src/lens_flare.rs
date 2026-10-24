//! Safe pure LensFlare (bright spot falloff core).
//!
//! Adds a bright spot that falls off with distance.
//! Works on Y800. No unwraps.
//!
//! Differential: the center is brighter than the edge.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlareError {
    WrongFormat,
}

pub struct LensFlare;

impl LensFlare {
    /// strength is 0..=255.
    pub fn apply(frame: &mut VideoFrame, cx: u32, cy: u32, strength: u8) -> Result<(), FlareError> {
        if frame.format != VideoFormat::Y800 {
            return Err(FlareError::WrongFormat);
        }
        let w = frame.width as f32;
        let h = frame.height as f32;
        let max_d = (w * w + h * h).sqrt();
        if let Some(plane) = frame.planes.get_mut(0) {
            for y in 0..frame.height {
                for x in 0..frame.width {
                    let dx = x as f32 - cx as f32;
                    let dy = y as f32 - cy as f32;
                    let d = (dx * dx + dy * dy).sqrt();
                    let t = 1.0 - (d / max_d).min(1.0);
                    let add = (strength as f32 * t) as u16;
                    let i = (y * frame.width + x) as usize;
                    if i < plane.len() {
                        let v = plane[i] as u16 + add;
                        plane[i] = v.min(255) as u8;
                    }
                }
            }
        }
        Ok(())
    }

    /// Read strength from live weights.
    pub fn from_weights(
        frame: &mut VideoFrame,
        cx: u32,
        cy: u32,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), FlareError> {
        let v = weights.get(name).unwrap_or(0.0);
        Self::apply(frame, cx, cy, v.clamp(0.0, 255.0) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        LensFlare::apply(&mut f, 4, 4, 100).unwrap();
        assert!(f.planes[0][4 * 8 + 4] > f.planes[0][0]);
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        let mut w = LiveWeights::new();
        w.set("flare.strength", 80.0).unwrap();
        LensFlare::from_weights(&mut f, 4, 4, &w, "flare.strength").unwrap();
        assert!(f.planes[0][4 * 8 + 4] > 0);
    }
}
