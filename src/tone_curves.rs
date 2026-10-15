//! Safe pure Curves (tone curve core).
//!
//! Remaps luminance through control points. Linear between
//! points. Works on Y800. No unwraps.
//!
//! Differential: a mid gray is lifted by an S-curve.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurvesError {
    WrongFormat,
    Empty,
}

pub struct Curves;

impl Curves {
    /// points are (input, output) pairs, 0..=255, sorted by input.
    pub fn apply(frame: &mut VideoFrame, points: &[(u8, u8)]) -> Result<(), CurvesError> {
        if frame.format != VideoFormat::Y800 {
            return Err(CurvesError::WrongFormat);
        }
        if points.is_empty() {
            return Err(CurvesError::Empty);
        }
        let lut = Self::build_lut(points);
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                *p = lut[*p as usize];
            }
        }
        Ok(())
    }

    fn build_lut(points: &[(u8, u8)]) -> [u8; 256] {
        let mut lut = [0u8; 256];
        let mut pts = points.to_vec();
        pts.sort_by_key(|p| p.0);
        if pts[0].0 != 0 {
            pts.insert(0, (0, pts[0].1));
        }
        if pts.last().unwrap().0 != 255 {
            pts.push((255, pts.last().unwrap().1));
        }
        for i in 0..256 {
            let x = i as u8;
            let mut out = pts[0].1;
            for w in pts.windows(2) {
                let (x0, y0) = w[0];
                let (x1, y1) = w[1];
                if x >= x0 && x <= x1 {
                    if x1 == x0 {
                        out = y1;
                    } else {
                        let t = (x - x0) as u32;
                        let span = (x1 - x0) as u32;
                        out = (y0 as u32 + (y1 as u32 - y0 as u32) * t / span) as u8;
                    }
                    break;
                }
            }
            lut[i] = out;
        }
        lut
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        assert!(Curves::apply(&mut f, &[]).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(128);
        Curves::apply(&mut f, &[(0, 0), (128, 180), (255, 255)]).unwrap();
        assert!(f.planes[0][0] > 128);
    }
}
