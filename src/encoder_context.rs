//! Safe pure EncoderContext (OBS encoder context core).
//!
//! The C version is the shared context every encoder carries:
//! name, settings, and a codec id. The safe path owns those pieces.
//! Empty names are rejected. No unwraps.
//!
//! Differential: create + settings round-trip.

use crate::obs_data::ObsData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncoderError {
    EmptyName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderType {
    Video,
    Audio,
}

#[derive(Clone, Debug)]
pub struct EncoderContext {
    name: String,
    codec: String,
    ty: EncoderType,
    settings: ObsData,
}

impl EncoderContext {
    /// Create an encoder context. Empty names are rejected.
    pub fn new(name: &str, codec: &str, ty: EncoderType) -> Result<Self, EncoderError> {
        if name.is_empty() {
            return Err(EncoderError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            codec: codec.to_string(),
            ty,
            settings: ObsData::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn codec(&self) -> &str {
        &self.codec
    }

    pub fn encoder_type(&self) -> EncoderType {
        self.ty
    }

    pub fn settings(&self) -> &ObsData {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ObsData {
        &mut self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let e = EncoderContext::new("h264", "obs_x264", EncoderType::Video).unwrap();
        assert_eq!(e.name(), "h264");
        assert_eq!(e.codec(), "obs_x264");
        assert_eq!(e.encoder_type(), EncoderType::Video);
    }

    #[test]
    fn empty_name() {
        assert!(EncoderContext::new("", "c", EncoderType::Audio).is_err());
    }

    #[test]
    fn differential() {
        let mut e = EncoderContext::new("aac", "ffmpeg_aac", EncoderType::Audio).unwrap();
        e.settings_mut().set_int("bitrate", 160).unwrap();
        assert_eq!(e.settings().get_int("bitrate").unwrap(), 160);
        assert_eq!(e.encoder_type(), EncoderType::Audio);
    }
}
