//! Safe pure Core (OBS core root).
//!
//! Owns the registries for sources, outputs, encoders, services,
//! scenes, and filters. Empty names are rejected by the registries.
//! No unwraps.
//!
//! Differential: add a source through the core and read it back.

use crate::encoder_registry::EncoderRegistry;
use crate::filter_registry::FilterRegistry;
use crate::module_registry::ModuleRegistry;
use crate::output_registry::OutputRegistry;
use crate::scene_registry::SceneRegistry;
use crate::service_registry::ServiceRegistry;
use crate::source_registry::SourceRegistry;

#[derive(Clone, Debug, Default)]
pub struct Core {
    sources: SourceRegistry,
    outputs: OutputRegistry,
    encoders: EncoderRegistry,
    services: ServiceRegistry,
    scenes: SceneRegistry,
    filters: FilterRegistry,
    modules: ModuleRegistry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    TypeNotRegistered,
    Duplicate,
    EmptyName,
}

impl Core {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a source only if its type id is registered by a loaded module.
    pub fn create_source(&mut self, name: &str, type_id: &str) -> Result<(), CoreError> {
        if name.is_empty() {
            return Err(CoreError::EmptyName);
        }
        if !self.modules.has_source_type(type_id) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.sources.add(name).map_err(|_| CoreError::Duplicate)
    }

    /// Create a filter only if its type id is registered by a loaded module.
    pub fn create_filter(&mut self, name: &str, type_id: &str) -> Result<(), CoreError> {
        if name.is_empty() {
            return Err(CoreError::EmptyName);
        }
        if !self.modules.has_filter_type(type_id) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.filters.add(name, type_id).map_err(|_| CoreError::Duplicate)
    }

    pub fn sources(&self) -> &SourceRegistry {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut SourceRegistry {
        &mut self.sources
    }

    pub fn outputs(&self) -> &OutputRegistry {
        &self.outputs
    }

    pub fn outputs_mut(&mut self) -> &mut OutputRegistry {
        &mut self.outputs
    }

    pub fn encoders(&self) -> &EncoderRegistry {
        &self.encoders
    }

    pub fn encoders_mut(&mut self) -> &mut EncoderRegistry {
        &mut self.encoders
    }

    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    pub fn scenes(&self) -> &SceneRegistry {
        &self.scenes
    }

    pub fn scenes_mut(&mut self) -> &mut SceneRegistry {
        &mut self.scenes
    }

    pub fn filters(&self) -> &FilterRegistry {
        &self.filters
    }

    pub fn filters_mut(&mut self) -> &mut FilterRegistry {
        &mut self.filters
    }

    pub fn modules(&self) -> &ModuleRegistry {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.modules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn add_source() {
        let mut c = Core::new();
        c.sources_mut().add("Camera").unwrap();
        assert_eq!(c.sources().get("Camera").unwrap().name(), "Camera");
    }

    #[test]
    fn differential() {
        let mut c = Core::new();
        c.sources_mut().add("cam").unwrap();
        c.outputs_mut().add("stream").unwrap();
        c.encoders_mut().add("h264", "obs_x264", EncoderType::Video).unwrap();
        c.services_mut().add("Twitch", "rtmp_common").unwrap();
        c.scenes_mut().add("Main").unwrap();
        c.filters_mut().add("blur", "gpu_blur").unwrap();

        c.sources_mut().get_mut("cam").unwrap().settings_mut().set_int("w", 1920).unwrap();
        assert_eq!(c.sources().get("cam").unwrap().settings().get_int("w").unwrap(), 1920);
        assert_eq!(c.outputs().get("stream").unwrap().name(), "stream");
        assert_eq!(c.scenes().get("Main").unwrap().len(), 0);
    }

    #[test]
    fn create_requires_registered_type() {
        let mut c = Core::new();
        c.modules_mut().add("obs-ffmpeg").unwrap();
        c.modules_mut().get_mut("obs-ffmpeg").unwrap().register_source("ffmpeg_source").unwrap();
        c.modules_mut().get_mut("obs-ffmpeg").unwrap().register_filter("gain_filter").unwrap();

        assert!(c.create_source("cam", "ffmpeg_source").is_ok());
        assert!(matches!(c.create_source("cam2", "missing"), Err(CoreError::TypeNotRegistered)));
        assert!(c.create_filter("g", "gain_filter").is_ok());
        assert_eq!(c.sources().get("cam").unwrap().name(), "cam");
    }
}
