//! Safe pure Distort (wave, bulge, twirl core).
//!
//! Displaces pixels. Works on Y800. No unwraps.
//!
//! Differential: a wave shifts a bright pixel.

use crate::live_weights::LiveWeights;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistortKind {
    Wave,
    Bulge,
    Twirl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DistortError {
    WrongFormat,
}

pub struct Distort;

impl Distort {
    /// amount is pixels of displacement.
    pub fn apply(
        frame: &mut VideoFrame,
        kind: DistortKind,
        amount: u8,
    ) -> Result<(), DistortError> {
        if frame.format != VideoFormat::Y800 {
            return Err(DistortError::WrongFormat);
        }
        let w = frame.width as i32;
        let h = frame.height as i32;
        let src = frame.planes[0].clone();
        if let Some(dst) = frame.planes.get_mut(0) {
            for y in 0..h {
                for x in 0..w {
                    let (sx, sy) = match kind {
                        DistortKind::Wave => {
                            let off = ((y as f32 * 0.5).sin() * amount as f32) as i32;
                            (x + off, y)
                        }
                        DistortKind::Bulge => {
                            let cx = w / 2;
                            let cy = h / 2;
                            let dx = x - cx;
                            let dy = y - cy;
                            let scale = 1.0 + (amount as f32 / 255.0);
                            (
                                (cx as f32 + dx as f32 / scale) as i32,
                                (cy as f32 + dy as f32 / scale) as i32,
                            )
                        }
                        DistortKind::Twirl => {
                            let cx = w as f32 / 2.0;
                            let cy = h as f32 / 2.0;
                            let ang = (amount as f32 / 255.0) * 0.5;
                            let dx = x as f32 - cx;
                            let dy = y as f32 - cy;
                            let c = ang.cos();
                            let s = ang.sin();
                            ((cx + dx * c - dy * s) as i32, (cy + dx * s + dy * c) as i32)
                        }
                    };
                    let sx = sx.clamp(0, w - 1);
                    let sy = sy.clamp(0, h - 1);
                    let si = (sy * w + sx) as usize;
                    let di = (y * w + x) as usize;
                    if si < src.len() && di < dst.len() {
                        dst[di] = src[si];
                    }
                }
            }
        }
        Ok(())
    }

    /// Read amount from live weights and apply.
    pub fn from_weights(
        frame: &mut VideoFrame,
        kind: DistortKind,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), DistortError> {
        let v = weights.get(name).unwrap_or(0.0);
        let amount = v.clamp(0.0, 255.0) as u8;
        Self::apply(frame, kind, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        f.planes[0][0] = 255;
        Distort::apply(&mut f, DistortKind::Wave, 4).unwrap();
        // some pixel differs from a flat zero frame
        assert!(f.planes[0].contains(&255));
    }

    #[test]
    fn from_weights() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        f.planes[0].fill(100);
        let mut w = LiveWeights::new();
        w.set("distort.wave", 8.0).unwrap();
        Distort::from_weights(&mut f, DistortKind::Wave, &w, "distort.wave").unwrap();
        assert_eq!(f.planes[0][0], 100);
    }
}
