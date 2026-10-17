//! Safe pure DodgeBurnSponge (tone tools core).
//!
//! Dodge lightens, burn darkens, sponge shifts toward
//! extremes. Works on Y800. No unwraps.
//!
//! Differential: dodge lifts, burn darkens, sponge pushes.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToneTool {
    Dodge,
    Burn,
    Sponge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToneError {
    WrongFormat,
}

pub struct DodgeBurnSponge;

impl DodgeBurnSponge {
    /// amount is 0..=100.
    pub fn apply(frame: &mut VideoFrame, tool: ToneTool, amount: u8) -> Result<(), ToneError> {
        if frame.format != VideoFormat::Y800 {
            return Err(ToneError::WrongFormat);
        }
        let a = amount.min(100) as i16;
        if let Some(plane) = frame.planes.get_mut(0) {
            for p in plane.iter_mut() {
                let v = *p as i16;
                let out = match tool {
                    ToneTool::Dodge => v + a,
                    ToneTool::Burn => v - a,
                    ToneTool::Sponge => {
                        if v >= 128 {
                            v + a / 2
                        } else {
                            v - a / 2
                        }
                    }
                };
                *p = out.clamp(0, 255) as u8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(128);
        DodgeBurnSponge::apply(&mut f, ToneTool::Dodge, 20).unwrap();
        assert_eq!(f.planes[0][0], 148);
        DodgeBurnSponge::apply(&mut f, ToneTool::Burn, 20).unwrap();
        assert_eq!(f.planes[0][0], 128);
        f.planes[0].fill(100);
        DodgeBurnSponge::apply(&mut f, ToneTool::Sponge, 20).unwrap();
        assert!(f.planes[0][0] < 100);
    }
}
