//! Safe pure Core (OBS core root).
//!
//! Owns the registries for sources, outputs, encoders, services,
//! scenes, and filters. Empty names are rejected by the registries.
//! No unwraps.
//!
//! Differential: add a source through the core and read it back.

use crate::collection_registry::CollectionRegistry;
use crate::display_registry::DisplayRegistry;
use crate::monitor_registry::MonitorRegistry;
use crate::recording_registry::RecordingRegistry;
use crate::replay_registry::ReplayRegistry;
use crate::canvas_registry::CanvasRegistry;
use crate::profile_registry::ProfileRegistry;
use crate::streaming_registry::StreamingRegistry;
use crate::virtual_cam_registry::VirtualCamRegistry;
use crate::transition_registry::TransitionRegistry;
use crate::encoder_registry::EncoderRegistry;
use crate::hotkey_registry::HotkeyRegistry;
use crate::filter_registry::FilterRegistry;
use crate::module_registry::ModuleRegistry;
use crate::output_registry::OutputRegistry;
use crate::scene_registry::SceneRegistry;
use crate::service_registry::ServiceRegistry;
use crate::source_registry::SourceRegistry;
use crate::pixel_canvas::PixelCanvas;
use crate::preview::{Preview, PreviewFrame};
use crate::session::Session;

#[derive(Clone, Debug, Default)]
pub struct Core {
    sources: SourceRegistry,
    outputs: OutputRegistry,
    encoders: EncoderRegistry,
    services: ServiceRegistry,
    scenes: SceneRegistry,
    filters: FilterRegistry,
    modules: ModuleRegistry,
    displays: DisplayRegistry,
    hotkeys: HotkeyRegistry,
    collections: CollectionRegistry,
    transitions: TransitionRegistry,
    current_scene: Option<String>,
    program_scene: Option<String>,
    preview_scene: Option<String>,
    monitors: MonitorRegistry,
    recordings: RecordingRegistry,
    streams: StreamingRegistry,
    replays: ReplayRegistry,
    virtual_cams: VirtualCamRegistry,
    profiles: ProfileRegistry,
    current_profile: Option<String>,
    canvases: CanvasRegistry,
    session: Session,
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
        self.sources.add_with_type(name, type_id).map_err(|_| CoreError::Duplicate)
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

    /// Load a module by id. Empty and duplicate ids are rejected.
    pub fn load_module(&mut self, id: &str) -> Result<(), CoreError> {
        self.modules.add(id).map_err(|e| match e {
            crate::module_registry::ModuleRegistryError::EmptyId => CoreError::EmptyName,
            crate::module_registry::ModuleRegistryError::Duplicate => CoreError::Duplicate,
            crate::module_registry::ModuleRegistryError::Missing => CoreError::TypeNotRegistered,
        })
    }

    /// Register a source type on a loaded module.
    pub fn register_source_type(&mut self, module_id: &str, type_id: &str) -> Result<(), CoreError> {
        let m = self.modules.get_mut(module_id).map_err(|_| CoreError::TypeNotRegistered)?;
        m.register_source(type_id).map_err(|e| match e {
            crate::module::ModuleError::EmptyId => CoreError::EmptyName,
            crate::module::ModuleError::Duplicate => CoreError::Duplicate,
        })
    }

    /// Register a filter type on a loaded module.
    pub fn register_filter_type(&mut self, module_id: &str, type_id: &str) -> Result<(), CoreError> {
        let m = self.modules.get_mut(module_id).map_err(|_| CoreError::TypeNotRegistered)?;
        m.register_filter(type_id).map_err(|e| match e {
            crate::module::ModuleError::EmptyId => CoreError::EmptyName,
            crate::module::ModuleError::Duplicate => CoreError::Duplicate,
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    /// Preview the core's session at a time.
    pub fn preview(&self, time: f32, canvas: &mut PixelCanvas) -> PreviewFrame {
        Preview::show(&self.session, time, canvas)
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

    pub fn displays(&self) -> &DisplayRegistry {
        &self.displays
    }

    pub fn displays_mut(&mut self) -> &mut DisplayRegistry {
        &mut self.displays
    }

    pub fn hotkeys(&self) -> &HotkeyRegistry {
        &self.hotkeys
    }

    pub fn hotkeys_mut(&mut self) -> &mut HotkeyRegistry {
        &mut self.hotkeys
    }

    pub fn collections(&self) -> &CollectionRegistry {
        &self.collections
    }

    pub fn collections_mut(&mut self) -> &mut CollectionRegistry {
        &mut self.collections
    }

    pub fn transitions(&self) -> &TransitionRegistry {
        &self.transitions
    }

    pub fn transitions_mut(&mut self) -> &mut TransitionRegistry {
        &mut self.transitions
    }

    /// Set the current scene. Returns TypeNotRegistered if the scene does not exist.
    pub fn set_current_scene(&mut self, name: &str) -> Result<(), CoreError> {
        if !self.scenes.contains(name) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.current_scene = Some(name.to_string());
        Ok(())
    }

    pub fn current_scene(&self) -> Option<&str> {
        self.current_scene.as_deref()
    }

    /// Set the program scene. Returns TypeNotRegistered if the scene does not exist.
    pub fn set_program_scene(&mut self, name: &str) -> Result<(), CoreError> {
        if !self.scenes.contains(name) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.program_scene = Some(name.to_string());
        Ok(())
    }

    pub fn program_scene(&self) -> Option<&str> {
        self.program_scene.as_deref()
    }

    /// Set the preview scene. Returns TypeNotRegistered if the scene does not exist.
    pub fn set_preview_scene(&mut self, name: &str) -> Result<(), CoreError> {
        if !self.scenes.contains(name) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.preview_scene = Some(name.to_string());
        Ok(())
    }

    pub fn preview_scene(&self) -> Option<&str> {
        self.preview_scene.as_deref()
    }

    pub fn monitors(&self) -> &MonitorRegistry {
        &self.monitors
    }

    pub fn monitors_mut(&mut self) -> &mut MonitorRegistry {
        &mut self.monitors
    }

    pub fn recordings(&self) -> &RecordingRegistry {
        &self.recordings
    }

    pub fn recordings_mut(&mut self) -> &mut RecordingRegistry {
        &mut self.recordings
    }

    pub fn streams(&self) -> &StreamingRegistry {
        &self.streams
    }

    pub fn streams_mut(&mut self) -> &mut StreamingRegistry {
        &mut self.streams
    }

    pub fn replays(&self) -> &ReplayRegistry {
        &self.replays
    }

    pub fn replays_mut(&mut self) -> &mut ReplayRegistry {
        &mut self.replays
    }

    pub fn virtual_cams(&self) -> &VirtualCamRegistry {
        &self.virtual_cams
    }

    pub fn virtual_cams_mut(&mut self) -> &mut VirtualCamRegistry {
        &mut self.virtual_cams
    }

    pub fn profiles(&self) -> &ProfileRegistry {
        &self.profiles
    }

    pub fn profiles_mut(&mut self) -> &mut ProfileRegistry {
        &mut self.profiles
    }

    /// Set the current profile. Returns TypeNotRegistered if it does not exist.
    pub fn set_current_profile(&mut self, name: &str) -> Result<(), CoreError> {
        if !self.profiles.contains(name) {
            return Err(CoreError::TypeNotRegistered);
        }
        self.current_profile = Some(name.to_string());
        Ok(())
    }

    pub fn current_profile(&self) -> Option<&str> {
        self.current_profile.as_deref()
    }

    pub fn canvases(&self) -> &CanvasRegistry {
        &self.canvases
    }

    pub fn canvases_mut(&mut self) -> &mut CanvasRegistry {
        &mut self.canvases
    }

    /// Swap program and preview. No-op if either is unset.
    pub fn transition_to_preview(&mut self) -> bool {
        if self.program_scene.is_some() && self.preview_scene.is_some() {
            std::mem::swap(&mut self.program_scene, &mut self.preview_scene);
            true
        } else {
            false
        }
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
        assert_eq!(c.sources().get("cam").unwrap().type_id(), "ffmpeg_source");
        assert_eq!(c.filters().get("g").unwrap().type_id(), "gain_filter");
    }

    #[test]
    fn current_scene() {
        let mut c = Core::new();
        c.scenes_mut().add("Main").unwrap();
        c.set_current_scene("Main").unwrap();
        assert_eq!(c.current_scene(), Some("Main"));
        assert!(matches!(c.set_current_scene("missing"), Err(CoreError::TypeNotRegistered)));
    }

    #[test]
    fn program_preview() {
        let mut c = Core::new();
        c.scenes_mut().add("Main").unwrap();
        c.scenes_mut().add("Overlay").unwrap();
        c.set_program_scene("Main").unwrap();
        c.set_preview_scene("Overlay").unwrap();
        assert_eq!(c.program_scene(), Some("Main"));
        assert_eq!(c.preview_scene(), Some("Overlay"));
        assert!(matches!(c.set_program_scene("missing"), Err(CoreError::TypeNotRegistered)));
    }

    #[test]
    fn transition_swaps() {
        let mut c = Core::new();
        c.scenes_mut().add("Main").unwrap();
        c.scenes_mut().add("Overlay").unwrap();
        c.set_program_scene("Main").unwrap();
        c.set_preview_scene("Overlay").unwrap();
        assert!(c.transition_to_preview());
        assert_eq!(c.program_scene(), Some("Overlay"));
        assert_eq!(c.preview_scene(), Some("Main"));
    }

    #[test]
    fn current_profile() {
        let mut c = Core::new();
        c.profiles_mut().add("Default").unwrap();
        c.set_current_profile("Default").unwrap();
        assert_eq!(c.current_profile(), Some("Default"));
        assert!(matches!(c.set_current_profile("missing"), Err(CoreError::TypeNotRegistered)));
    }
}
