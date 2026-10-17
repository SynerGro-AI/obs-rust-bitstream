//! Safe pure ChannelMixer (single-channel scale core).
//!
//! On Y800 there is one channel. The mixer scales it
//! by a percentage and clamps. No unwraps.
//!
//! Differential: 50% of 200 is 100.

use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MixerError {
    WrongFormat,
}

pub struct ChannelMixer;

impl ChannelMixer {
    /// percent is 0..=200. 100 is unchanged.
    pub fn apply(frame: &mut VideoFrame, percent: u8) -> Result<(), MixerError> {
        if frame.format != VideoFormat::Y800 {
            return Err(MixerError::WrongFormat);
        }
        let p = percent.min(200) as u16;
        if let Some(plane) = frame.planes.get_mut(0) {
            for px in plane.iter_mut() {
                let v = (*px as u16 * p) / 100;
                *px = v.min(255) as u8;
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
        f.planes[0].fill(200);
        ChannelMixer::apply(&mut f, 50).unwrap();
        assert_eq!(f.planes[0][0], 100);
    }
}
