//! Safe pure EncoderRegistry (OBS encoder lookup core).
//!
//! Name-to-EncoderContext map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + get returns the same name; settings round-trip.

use std::collections::HashMap;

use crate::encoder_context::{EncoderContext, EncoderError, EncoderType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncoderRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<EncoderError> for EncoderRegistryError {
    fn from(_: EncoderError) -> Self {
        EncoderRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct EncoderRegistry {
    encoders: HashMap<String, EncoderContext>,
}

impl EncoderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        name: &str,
        codec: &str,
        ty: EncoderType,
    ) -> Result<(), EncoderRegistryError> {
        if name.is_empty() {
            return Err(EncoderRegistryError::EmptyName);
        }
        if self.encoders.contains_key(name) {
            return Err(EncoderRegistryError::Duplicate);
        }
        let ctx = EncoderContext::new(name, codec, ty)?;
        self.encoders.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&EncoderContext, EncoderRegistryError> {
        self.encoders.get(name).ok_or(EncoderRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut EncoderContext, EncoderRegistryError> {
        self.encoders
            .get_mut(name)
            .ok_or(EncoderRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.encoders.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.encoders.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.encoders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = EncoderRegistry::new();
        r.add("h264", "obs_x264", EncoderType::Video).unwrap();
        assert_eq!(r.get("h264").unwrap().codec(), "obs_x264");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = EncoderRegistry::new();
        r.add("a", "c", EncoderType::Audio).unwrap();
        assert!(matches!(
            r.add("a", "c", EncoderType::Audio),
            Err(EncoderRegistryError::Duplicate)
        ));
        assert!(matches!(
            r.add("", "c", EncoderType::Audio),
            Err(EncoderRegistryError::EmptyName)
        ));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = EncoderRegistry::new();
        assert!(matches!(r.get("nope"), Err(EncoderRegistryError::Missing)));
        r.add("e", "c", EncoderType::Video).unwrap();
        assert!(r.remove("e"));
        assert!(!r.contains("e"));
    }

    #[test]
    fn differential() {
        let mut r = EncoderRegistry::new();
        r.add("aac", "ffmpeg_aac", EncoderType::Audio).unwrap();
        r.get_mut("aac")
            .unwrap()
            .settings_mut()
            .set_int("bitrate", 160)
            .unwrap();
        assert_eq!(
            r.get("aac").unwrap().settings().get_int("bitrate").unwrap(),
            160
        );
    }
}
