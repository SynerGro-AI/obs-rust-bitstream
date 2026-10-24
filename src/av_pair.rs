//! Safe pure AvPair (video + audio output info).
//!
//! Checks that a VideoOutputInfo and AudioOutputInfo
//! both have non-empty names and positive rates/sizes.
//! No unwraps.
//!
//! Differential: a pair with more pixels reports more.

use crate::audio_output_info::AudioOutputInfo;
use crate::video_output_info::VideoOutputInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvPair {
    pub video_pixels: u64,
    pub audio_rate: u32,
    pub audio_channels: u32,
}

impl AvPair {
    /// Build from the two output infos.
    pub fn from(video: &VideoOutputInfo, audio: &AudioOutputInfo) -> Self {
        Self {
            video_pixels: video.pixels(),
            audio_rate: audio.rate(),
            audio_channels: audio.channels(),
        }
    }

    /// True if both sides are non-zero.
    pub fn is_valid(&self) -> bool {
        self.video_pixels > 0 && self.audio_rate > 0 && self.audio_channels > 0
    }

    /// One line: "pixels=N rate=N channels=N"
    pub fn one_line(&self) -> String {
        format!(
            "pixels={} rate={} channels={}",
            self.video_pixels, self.audio_rate, self.audio_channels
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_format::{AudioFormat, SpeakerLayout};
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn valid() {
        let v = VideoOutputInfo::new("main", VideoFormat::Y800, 30, 1, 1920, 1080).unwrap();
        let a =
            AudioOutputInfo::new("main", 48000, AudioFormat::F32, SpeakerLayout::Stereo).unwrap();
        let p = AvPair::from(&v, &a);
        assert!(p.is_valid());
        assert_eq!(p.video_pixels, 1920 * 1080);
        assert_eq!(p.audio_channels, 2);
    }

    #[test]
    fn one_line() {
        let v = VideoOutputInfo::new("main", VideoFormat::Y800, 30, 1, 640, 480).unwrap();
        let a = AudioOutputInfo::new("main", 44100, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let p = AvPair::from(&v, &a);
        assert!(p.one_line().contains("rate=44100"));
    }

    #[test]
    fn differential() {
        let va = VideoOutputInfo::new("a", VideoFormat::Y800, 30, 1, 640, 480).unwrap();
        let vb = VideoOutputInfo::new("b", VideoFormat::Y800, 30, 1, 1920, 1080).unwrap();
        let a = AudioOutputInfo::new("a", 48000, AudioFormat::S16, SpeakerLayout::Mono).unwrap();
        let pa = AvPair::from(&va, &a);
        let pb = AvPair::from(&vb, &a);
        assert!(pb.video_pixels > pa.video_pixels);
    }
}
