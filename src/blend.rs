//! Safe pure BlendMode (layer blend core).
//!
//! Combines two Y800 frames with a blend mode.
//! Sizes must match. No unwraps.
//!
//! Differential: multiply darkens, screen lightens.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlendError {
    WrongFormat,
    SizeMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Difference,
}

pub struct Blend;

impl Blend {
    pub fn apply(
        base: &mut VideoFrame,
        over: &VideoFrame,
        mode: BlendMode,
    ) -> Result<(), BlendError> {
        if base.format != VideoFormat::Y800 || over.format != VideoFormat::Y800 {
            return Err(BlendError::WrongFormat);
        }
        if base.width != over.width || base.height != over.height {
            return Err(BlendError::SizeMismatch);
        }
        if let (Some(dst), Some(src)) = (base.planes.get_mut(0), over.planes.first()) {
            for (d, s) in dst.iter_mut().zip(src.iter()) {
                *d = Self::pixel(*d, *s, mode);
            }
        }
        Ok(())
    }

    fn pixel(a: u8, b: u8, mode: BlendMode) -> u8 {
        let af = a as u16;
        let bf = b as u16;
        match mode {
            BlendMode::Normal => b,
            BlendMode::Multiply => ((af * bf) / 255) as u8,
            BlendMode::Screen => (255 - ((255 - af) * (255 - bf) / 255)) as u8,
            BlendMode::Overlay => {
                if af < 128 {
                    ((2 * af * bf) / 255) as u8
                } else {
                    (255 - (2 * (255 - af) * (255 - bf) / 255)) as u8
                }
            }
            BlendMode::Darken => a.min(b),
            BlendMode::Lighten => a.max(b),
            BlendMode::Difference => a.abs_diff(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut base = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        base.planes[0].fill(200);
        let mut over = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        over.planes[0].fill(100);
        Blend::apply(&mut base, &over, BlendMode::Multiply).unwrap();
        assert!(base.planes[0][0] < 200);
        let mut base2 = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        base2.planes[0].fill(100);
        Blend::apply(&mut base2, &over, BlendMode::Screen).unwrap();
        assert!(base2.planes[0][0] > 100);
    }
}
