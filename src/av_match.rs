//! Safe pure AvMatch (pair → scale + convert).
//!
//! Checks whether an AvPair matches a VideoScaleInfo and
//! AudioConvertInfo. Returns the pixel and rate deltas.
//! No unwraps.
//!
//! Differential: a larger target produces a larger delta.

use crate::audio_convert_info::AudioConvertInfo;
use crate::av_pair::AvPair;
use crate::video_scale_info::VideoScaleInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvMatch {
    pub pixel_delta: u64,
    pub rate_delta: u32,
    pub matches: bool,
}

impl AvMatch {
    /// Compare the pair to the scale and convert requests.
    pub fn from(pair: &AvPair, scale: &VideoScaleInfo, convert: &AudioConvertInfo) -> Self {
        let pixel_delta = pair.video_pixels.abs_diff(scale.pixels());
        let rate_delta = pair.audio_rate.abs_diff(convert.rate());
        Self {
            pixel_delta,
            rate_delta,
            matches: pixel_delta == 0 && rate_delta == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_format::{AudioFormat, SpeakerLayout};
    use crate::audio_output_info::AudioOutputInfo;
    use crate::video_fourcc::VideoFormat;
    use crate::video_output_info::VideoOutputInfo;

    fn pair(w: u32, h: u32, rate: u32) -> AvPair {
        let v = VideoOutputInfo::new("v", VideoFormat::Y800, 30, 1, w, h).unwrap();
        let a = AudioOutputInfo::new("a", rate, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        AvPair::from(&v, &a)
    }

    #[test]
    fn match_same() {
        let p = pair(640, 480, 48000);
        let s = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        let m = AvMatch::from(&p, &s, &c);
        assert!(m.matches);
        assert_eq!(m.pixel_delta, 0);
    }

    #[test]
    fn differ() {
        let p = pair(640, 480, 44100);
        let s = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        let m = AvMatch::from(&p, &s, &c);
        assert!(!m.matches);
        assert!(m.pixel_delta > 0);
        assert_eq!(m.rate_delta, 3900);
    }

    #[test]
    fn differential() {
        let p = pair(320, 240, 44100);
        let sa = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let sb = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let ma = AvMatch::from(&p, &sa, &c);
        let mb = AvMatch::from(&p, &sb, &c);
        assert!(mb.pixel_delta > ma.pixel_delta);
    }
}
