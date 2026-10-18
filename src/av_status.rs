//! Safe pure AvStatus (pair + match report).
//!
//! One line of pixels, rate, channels, and match.
//! No unwraps.
//!
//! Differential: a mismatch reports matches=false.

use crate::audio_convert_info::AudioConvertInfo;
use crate::av_match::AvMatch;
use crate::av_pair::AvPair;
use crate::video_scale_info::VideoScaleInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvStatus {
    pub pixels: u64,
    pub rate: u32,
    pub channels: u32,
    pub matches: bool,
}

impl AvStatus {
    /// Build from the pair and the scale/convert requests.
    pub fn from(pair: &AvPair, scale: &VideoScaleInfo, convert: &AudioConvertInfo) -> Self {
        let m = AvMatch::from(pair, scale, convert);
        Self {
            pixels: pair.video_pixels,
            rate: pair.audio_rate,
            channels: pair.audio_channels,
            matches: m.matches,
        }
    }

    /// One line: "pixels=N rate=N channels=N matches=true/false"
    pub fn one_line(&self) -> String {
        format!(
            "pixels={} rate={} channels={} matches={}",
            self.pixels, self.rate, self.channels, self.matches
        )
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
    fn reports_match() {
        let p = pair(640, 480, 48000);
        let s = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        let st = AvStatus::from(&p, &s, &c);
        assert!(st.matches);
        assert!(st.one_line().contains("matches=true"));
    }

    #[test]
    fn reports_mismatch() {
        let p = pair(640, 480, 44100);
        let s = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Stereo).unwrap();
        let st = AvStatus::from(&p, &s, &c);
        assert!(!st.matches);
    }

    #[test]
    fn differential() {
        let p = pair(320, 240, 44100);
        let sa = VideoScaleInfo::new(VideoFormat::Y800, 640, 480).unwrap();
        let sb = VideoScaleInfo::new(VideoFormat::Y800, 1920, 1080).unwrap();
        let c = AudioConvertInfo::new(48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let a = AvStatus::from(&p, &sa, &c);
        let b = AvStatus::from(&p, &sb, &c);
        assert!(!a.matches);
        assert!(!b.matches);
        assert_eq!(a.pixels, b.pixels);
    }
}
