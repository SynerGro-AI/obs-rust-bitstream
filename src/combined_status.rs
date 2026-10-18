//! Safe pure CombinedStatus (video + audio one-liner).
//!
//! Builds a single status line from the active video source
//! kind and the active audio source name. Empty names are
//! shown as "-". No unwraps.
//!
//! Differential: different sources produce different lines.

use crate::audio_source_list::AudioSourceList;
use crate::capture_select::CaptureSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombinedStatus {
    pub video: String,
    pub audio: String,
}

impl CombinedStatus {
    /// Build from a video source and an audio source list.
    pub fn from(video: &CaptureSource, audio: &AudioSourceList) -> Self {
        let video_kind = video.kind().to_string();
        let audio_name = audio.active_name().unwrap_or("-").to_string();
        Self {
            video: video_kind,
            audio: audio_name,
        }
    }

    /// One line: "video=<kind> audio=<name>"
    pub fn one_line(&self) -> String {
        format!("video={} audio={}", self.video, self.audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;

    #[test]
    fn synthetic_line() {
        let video = CaptureSource::synthetic(64, 64);
        let mut audio = AudioSourceList::new();
        audio.add("mic", AudioSource::synthetic(48000, 1));
        let status = CombinedStatus::from(&video, &audio);
        assert_eq!(status.one_line(), "video=synthetic audio=mic");
    }

    #[test]
    fn empty_audio() {
        let video = CaptureSource::synthetic(32, 32);
        let audio = AudioSourceList::new();
        let status = CombinedStatus::from(&video, &audio);
        assert_eq!(status.one_line(), "video=synthetic audio=-");
    }

    #[test]
    fn differential() {
        let video = CaptureSource::synthetic(16, 16);
        let mut a = AudioSourceList::new();
        a.add("mic", AudioSource::synthetic(48000, 1));
        let mut b = AudioSourceList::new();
        b.add("line", AudioSource::synthetic(48000, 1));
        assert_ne!(
            CombinedStatus::from(&video, &a).one_line(),
            CombinedStatus::from(&video, &b).one_line()
        );
    }
}
