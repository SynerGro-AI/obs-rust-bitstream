//! Safe pure UvMap (vertex UV core).
//!
//! Holds UV coordinates per vertex. Can sample a
//! value from a Y800 frame. No unwraps.
//!
//! Differential: set UV + sample round-trips.

use crate::live_weights::LiveWeights;
use crate::video_frame::VideoFrame;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Uv {
    pub u: f32,
    pub v: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UvError {
    Missing,
    WrongFormat,
}

#[derive(Clone, Debug, Default)]
pub struct UvMap {
    coords: Vec<Uv>,
}

impl UvMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, index: usize, u: f32, v: f32) {
        if index >= self.coords.len() {
            self.coords.resize(index + 1, Uv { u: 0.0, v: 0.0 });
        }
        self.coords[index] = Uv {
            u: u.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
        };
    }

    pub fn get(&self, index: usize) -> Result<Uv, UvError> {
        self.coords.get(index).copied().ok_or(UvError::Missing)
    }

    pub fn len(&self) -> usize {
        self.coords.len()
    }

    /// Sample a Y800 frame at the UV for a vertex.
    pub fn sample(&self, index: usize, frame: &VideoFrame) -> Result<u8, UvError> {
        if frame.format != crate::video_fourcc::VideoFormat::Y800 {
            return Err(UvError::WrongFormat);
        }
        let uv = self.get(index)?;
        let x = (uv.u * (frame.width.saturating_sub(1)) as f32) as u32;
        let y = (uv.v * (frame.height.saturating_sub(1)) as f32) as u32;
        let i = (y * frame.width + x) as usize;
        frame
            .planes
            .first()
            .and_then(|p| p.get(i))
            .copied()
            .ok_or(UvError::Missing)
    }

    /// Read a UV from live weights.
    pub fn from_weights(&mut self, index: usize, weights: &LiveWeights) {
        let nu = format!("uv.{}.u", index);
        let nv = format!("uv.{}.v", index);
        let u = weights.get(&nu).unwrap_or(0.0);
        let v = weights.get(&nv).unwrap_or(0.0);
        if weights.get(&nu).is_ok() || weights.get(&nv).is_ok() {
            self.set(index, u, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn missing() {
        let m = UvMap::new();
        assert!(m.get(0).is_err());
    }

    #[test]
    fn differential() {
        let mut m = UvMap::new();
        m.set(0, 0.0, 0.0);
        m.set(1, 1.0, 1.0);
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0] = vec![10, 20, 30, 40];
        assert_eq!(m.sample(0, &f).unwrap(), 10);
        assert_eq!(m.sample(1, &f).unwrap(), 40);
        let mut w = LiveWeights::new();
        w.set("uv.0.u", 1.0).unwrap();
        m.from_weights(0, &w);
        assert_eq!(m.get(0).unwrap().u, 1.0);
    }
}
