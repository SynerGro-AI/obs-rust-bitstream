//! Safe pure OutputEncoder (OBS set video/audio encoder).
//!
//! Parsed from libobs/obs-output.c.
//! An output can hold one video encoder and up to
//! MAX_OUTPUT_AUDIO_ENCODERS audio encoders.
//! Only encoded outputs accept them. No unwraps.
//!
//! Differential: setting a video encoder changes the name.

use crate::encoder_context::{EncoderContext, EncoderType};
use crate::output_info::OutputInfo;

/// Matching OBS MAX_OUTPUT_AUDIO_ENCODERS.
pub const MAX_OUTPUT_AUDIO_ENCODERS: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputEncoderError {
    NotEncoded,
    WrongType,
    Index,
}

#[derive(Clone, Debug, Default)]
pub struct OutputEncoder {
    video: Option<EncoderContext>,
    audio: [Option<EncoderContext>; MAX_OUTPUT_AUDIO_ENCODERS],
}

impl OutputEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the video encoder. Only encoded outputs accept it.
    /// Returns error if the encoder is not video type.
    pub fn set_video(
        &mut self,
        output: &OutputInfo,
        encoder: EncoderContext,
    ) -> Result<(), OutputEncoderError> {
        if !output.is_encoded() {
            return Err(OutputEncoderError::NotEncoded);
        }
        if encoder.encoder_type() != EncoderType::Video {
            return Err(OutputEncoderError::WrongType);
        }
        self.video = Some(encoder);
        Ok(())
    }

    /// Set an audio encoder at index. Only encoded outputs accept it.
    pub fn set_audio(
        &mut self,
        output: &OutputInfo,
        index: usize,
        encoder: EncoderContext,
    ) -> Result<(), OutputEncoderError> {
        if !output.is_encoded() {
            return Err(OutputEncoderError::NotEncoded);
        }
        if encoder.encoder_type() != EncoderType::Audio {
            return Err(OutputEncoderError::WrongType);
        }
        if index >= MAX_OUTPUT_AUDIO_ENCODERS {
            return Err(OutputEncoderError::Index);
        }
        self.audio[index] = Some(encoder);
        Ok(())
    }

    pub fn video(&self) -> Option<&EncoderContext> {
        self.video.as_ref()
    }

    pub fn audio(&self, index: usize) -> Option<&EncoderContext> {
        self.audio.get(index).and_then(|e| e.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};

    #[test]
    fn set_video() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut oe = OutputEncoder::new();
        let enc = EncoderContext::new("h264", "obs_x264", EncoderType::Video).unwrap();
        oe.set_video(&out, enc).unwrap();
        assert_eq!(oe.video().unwrap().name(), "h264");
        let _ = out;
    }

    #[test]
    fn rejects_wrong_type() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut oe = OutputEncoder::new();
        let enc = EncoderContext::new("aac", "ffmpeg_aac", EncoderType::Audio).unwrap();
        assert!(oe.set_video(&out, enc).is_err());
    }

    #[test]
    fn rejects_unencoded() {
        let out = OutputInfo::new("raw", "Raw", OUTPUT_AV).unwrap();
        let mut oe = OutputEncoder::new();
        let enc = EncoderContext::new("h264", "obs_x264", EncoderType::Video).unwrap();
        assert!(oe.set_video(&out, enc).is_err());
    }

    #[test]
    fn audio_index() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut oe = OutputEncoder::new();
        let enc = EncoderContext::new("aac", "ffmpeg_aac", EncoderType::Audio).unwrap();
        oe.set_audio(&out, 0, enc).unwrap();
        assert!(oe.audio(0).is_some());
        assert!(oe.set_audio(&out, 6, EncoderContext::new("aac", "ffmpeg_aac", EncoderType::Audio).unwrap()).is_err());
    }

    #[test]
    fn differential() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut a = OutputEncoder::new();
        let b = OutputEncoder::new();
        a.set_video(&out, EncoderContext::new("h264", "obs_x264", EncoderType::Video).unwrap()).unwrap();
        assert!(a.video().is_some());
        assert!(b.video().is_none());
    }
}
