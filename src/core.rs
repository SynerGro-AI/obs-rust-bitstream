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
use crate::ai::AiAssistant;
use crate::audio_timeline::AudioTimeline;
use crate::effect_list::EffectList;
use crate::replay_save::{ReplaySave, SaveFormat};
use crate::instant_replay::InstantReplay;
use crate::sign_in::Session as SignIn;
use crate::organization::Organization;
use crate::file_cache::FileCache;
use crate::fingerprint::FingerprintStore;
use crate::schedule::Schedule;
use crate::macro_seq::Macro;
use crate::log::Log;
use crate::bind_profile::BindProfile;
use crate::keybind::Keybind;
use crate::frame_clock::FrameClock;
use crate::virtual_camera::VirtualCamera;
use crate::program_stream::ProgramStream;
use crate::program_record::ProgramRecord;
use crate::tally::Tally;
use crate::multiview::Multiview;
use crate::studio::Studio;
use crate::options_menu::OptionsMenu;
use crate::live_feed::LiveFeed;
use crate::keyer::Keyer;
use crate::live_camera::LiveCamera;
use crate::performance::Performance;
use crate::program::Program;
use crate::session::Session;
use crate::cue_sheet::CueSheet;
use crate::action::Action;
use crate::speech::Speech;
use crate::storyboard::Storyboard;
use crate::show_state::{ShowMode, ShowState};

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
    show: ShowState,
    cues: CueSheet,
    keyer: Keyer,
    board: Storyboard,
    speech: Speech,
    action: Action,
    effects: EffectList,
    feed: LiveFeed,
    menu: OptionsMenu,
    studio: Studio,
    multiview: Multiview,
    tally: Tally,
    record: ProgramRecord,
    stream: ProgramStream,
    vcam: VirtualCamera,
    clock: FrameClock,
    replay: InstantReplay,
    saves: ReplaySave,
    binds: Keybind,
    bind_profile: BindProfile,
    log: Log,
    key_macros: Macro,
    timed: Schedule,
    fingerprints: FingerprintStore,
    cache: FileCache,
    team: Organization,
    signed_in: SignIn,
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

    pub fn show(&self) -> &ShowState {
        &self.show
    }

    pub fn show_mut(&mut self) -> &mut ShowState {
        &mut self.show
    }

    pub fn show_mode(&self) -> ShowMode {
        self.show.mode()
    }

    pub fn cues(&self) -> &CueSheet {
        &self.cues
    }

    pub fn cues_mut(&mut self) -> &mut CueSheet {
        &mut self.cues
    }

    pub fn keyer(&self) -> &Keyer {
        &self.keyer
    }

    pub fn keyer_mut(&mut self) -> &mut Keyer {
        &mut self.keyer
    }

    pub fn board(&self) -> &Storyboard {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Storyboard {
        &mut self.board
    }

    pub fn speech(&self) -> &Speech {
        &self.speech
    }

    pub fn speech_mut(&mut self) -> &mut Speech {
        &mut self.speech
    }

    pub fn last_action(&self) -> &Action {
        &self.action
    }

    pub fn last_action_mut(&mut self) -> &mut Action {
        &mut self.action
    }

    /// Detect an action and return the scene the director picks.
    pub fn see(&mut self, ai: &AiAssistant, action: &str) -> crate::ai_director::SceneKind {
        self.action.detect(action);
        crate::ai_director::AiDirector::pick(ai, action)
    }

    /// Hear a line and return the scene the director picks.
    pub fn hear(&mut self, ai: &AiAssistant, line: &str) -> crate::ai_director::SceneKind {
        self.speech.hear(line);
        crate::ai_director::AiDirector::pick(ai, line)
    }

    /// Advance the storyboard and return the scene the director picks.
    pub fn advance_beat(&mut self, ai: &crate::ai::AiAssistant) -> crate::ai_director::SceneKind {
        self.board.next();
        crate::ai_director::AiDirector::from_board(ai, &self.board)
    }

    /// Advance the rundown and set the show topic.
    pub fn advance_cue(&mut self) -> Option<String> {
        let topic = self.cues.next().map(|s| s.to_string());
        if let Some(ref t) = topic {
            self.show.set_topic(t);
        }
        topic
    }

    /// Preview the core's session at a time.
    pub fn preview(&self, time: f32, canvas: &mut PixelCanvas) -> PreviewFrame {
        Preview::show(&self.session, time, canvas)
    }

    /// Go live with a news camera and topic.
    pub fn go_live_news(
        &self,
        camera: &LiveCamera,
        topic: &str,
        canvas: &mut PixelCanvas,
    ) -> crate::output::OutputFrame {
        Program::live_news(camera, topic, canvas)
    }

    /// Go live with a performance.
    pub fn go_live_performance(
        &self,
        performance: &Performance,
        camera: &LiveCamera,
        canvas: &mut PixelCanvas,
    ) -> crate::output::OutputFrame {
        Program::live_performance(performance, camera, canvas)
    }

    /// Run the AI crew and go live with a performance.
    pub fn run_show(
        &self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        topic: &str,
        canvas: &mut PixelCanvas,
    ) -> crate::output::OutputFrame {
        Program::live_with_crew(ai, performance, camera, topic, canvas)
    }

    /// Run a full show using the current cue as the topic.
    pub fn run_full_show(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        audio: &AudioTimeline,
        time: f32,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, f32) {
        let topic = self.cues.current().unwrap_or("show").to_string();
        self.show.set_topic(&topic);
        self.show.go_live();
        Program::live_show(ai, performance, camera, audio, &topic, time, canvas)
    }

    /// Run a full show using the core's keyer.
    pub fn run_keyed_show(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        audio: &AudioTimeline,
        time: f32,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, f32) {
        let topic = self.cues.current().unwrap_or("show").to_string();
        self.show.set_topic(&topic);
        self.show.go_live();
        let _crew = crate::ai_operator::AiOperator::run(ai, camera, &topic);
        let frame = Program::live_performance_keyed(performance, camera, &self.keyer, canvas);
        let level = audio.render(time);
        (frame, level)
    }

    /// Run a directed show from the current storyboard beat.
    pub fn run_directed(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, crate::ai_director::SceneKind) {
        let line = self.board.current().unwrap_or("show").to_string();
        self.show.set_topic(&line);
        self.show.go_live();
        Program::live_directed(ai, performance, camera, &line, canvas)
    }

    /// Hear a line and render the scene the director picks.
    pub fn hear_and_show(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        line: &str,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, crate::ai_director::SceneKind) {
        self.speech.hear(line);
        self.show.set_topic(line);
        self.show.go_live();
        Program::live_directed(ai, performance, camera, line, canvas)
    }

    /// Hear a line, trigger its effect, and render the scene.
    pub fn hear_and_effect(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        line: &str,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, crate::ai_director::SceneKind) {
        let (frame, kind) = self.hear_and_show(ai, performance, camera, line, canvas);
        crate::ai_director::AiDirector::trigger(kind, &mut self.effects);
        (frame, kind)
    }

    /// See an action and render the scene the director picks.
    pub fn see_and_show(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        action: &str,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, crate::ai_director::SceneKind) {
        self.action.detect(action);
        self.show.set_topic(action);
        self.show.go_live();
        Program::live_directed(ai, performance, camera, action, canvas)
    }

    /// See an action, trigger its effect, and render the scene.
    pub fn see_and_effect(
        &mut self,
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        action: &str,
        canvas: &mut PixelCanvas,
    ) -> (crate::output::OutputFrame, crate::ai_director::SceneKind) {
        let (frame, kind) = self.see_and_show(ai, performance, camera, action, canvas);
        crate::ai_director::AiDirector::trigger(kind, &mut self.effects);
        (frame, kind)
    }

    pub fn live_effects(&self) -> &EffectList {
        &self.effects
    }

    pub fn feed(&self) -> &LiveFeed {
        &self.feed
    }

    pub fn feed_mut(&mut self) -> &mut LiveFeed {
        &mut self.feed
    }

    pub fn menu(&self) -> &OptionsMenu {
        &self.menu
    }

    pub fn menu_mut(&mut self) -> &mut OptionsMenu {
        &mut self.menu
    }

    pub fn studio_mode(&self) -> &Studio {
        &self.studio
    }

    pub fn studio_mode_mut(&mut self) -> &mut Studio {
        &mut self.studio
    }

    pub fn multiview(&self) -> &Multiview {
        &self.multiview
    }

    pub fn multiview_mut(&mut self) -> &mut Multiview {
        &mut self.multiview
    }

    pub fn tally(&self) -> &Tally {
        &self.tally
    }

    pub fn tally_mut(&mut self) -> &mut Tally {
        &mut self.tally
    }

    pub fn record(&self) -> &ProgramRecord {
        &self.record
    }

    pub fn record_mut(&mut self) -> &mut ProgramRecord {
        &mut self.record
    }

    pub fn stream(&self) -> &ProgramStream {
        &self.stream
    }

    pub fn stream_mut(&mut self) -> &mut ProgramStream {
        &mut self.stream
    }

    pub fn vcam(&self) -> &VirtualCamera {
        &self.vcam
    }

    pub fn vcam_mut(&mut self) -> &mut VirtualCamera {
        &mut self.vcam
    }

    /// Start the virtual camera.
    pub fn start_vcam(&mut self) {
        self.vcam.start();
    }

    /// Stop the virtual camera.
    pub fn stop_vcam(&mut self) {
        self.vcam.stop();
    }

    /// Send one frame to every running output. Returns how many received it.
    pub fn send_frame(&mut self) -> usize {
        crate::output_bus::OutputBus::send(&mut self.record, &mut self.stream, &mut self.vcam)
    }

    pub fn clock(&self) -> &FrameClock {
        &self.clock
    }

    /// Tick the clock, keep the frame, and send it. Returns the frame number.
    pub fn tick(&mut self) -> u64 {
        let _ = self.send_frame();
        let n = self.clock.tick();
        self.replay.push(n);
        n
    }

    pub fn replay(&self) -> &InstantReplay {
        &self.replay
    }

    pub fn saves(&self) -> &ReplaySave {
        &self.saves
    }

    /// Save the replay as a clip, gif, or video.
    pub fn save_replay(&mut self, name: &str, format: SaveFormat) -> Result<(), crate::replay_save::SaveError> {
        self.saves.save_as(name, &self.replay, format)
    }

    /// Export a saved clip to a file path.
    pub fn export(&self, name: &str, path: &str) -> Result<crate::export::Export, crate::export::ExportError> {
        crate::export::Export::write(&self.saves, name, path)
    }

    /// Export a saved clip and return its bytes.
    pub fn export_bytes(&self, name: &str, path: &str) -> Result<Vec<u8>, crate::export::ExportError> {
        let e = self.export(name, path)?;
        Ok(crate::file_writer::FileWriter::encode(&e))
    }

    /// Share an export to a target.
    pub fn share(&self, name: &str, path: &str, target: &str) -> Result<crate::share::Share, crate::share::ShareError> {
        let bytes = self.export_bytes(name, path).map_err(|_| crate::share::ShareError::EmptyPayload)?;
        crate::share::Share::post(target, &bytes)
    }

    /// Share an export and return the confirmation message.
    pub fn share_and_confirm(&self, name: &str, path: &str, target: &str) -> Result<String, crate::share::ShareError> {
        let s = self.share(name, path, target)?;
        crate::notify::Notify::confirm(&s).map_err(|_| crate::share::ShareError::EmptyTarget)
    }

    /// Current output state.
    pub fn status(&self) -> crate::status::Status {
        crate::status::Status::from(&self.record, &self.stream, &self.vcam)
    }

    pub fn binds(&self) -> &Keybind {
        &self.binds
    }

    pub fn binds_mut(&mut self) -> &mut Keybind {
        &mut self.binds
    }

    /// Bind the standard keys: r record, s stream, v vcam.
    pub fn bind_defaults(&mut self) {
        let _ = self.binds.bind("r", "record");
        let _ = self.binds.bind("s", "stream");
        let _ = self.binds.bind("v", "vcam");
        let _ = self.binds.bind("R", "stop_record");
        let _ = self.binds.bind("S", "stop_stream");
        let _ = self.binds.bind("V", "stop_vcam");
    }

    /// List the bound keys.
    pub fn help(&self) -> String {
        crate::help::Help::text(&self.binds)
    }

    /// Rebind a key to a new action.
    pub fn rebind(&mut self, key: &str, action: &str) -> Result<(), crate::keybind::KeybindError> {
        self.binds.rebind(key, action)
    }

    /// Remove a key binding.
    pub fn unbind(&mut self, key: &str) {
        self.binds.unbind(key);
    }

    /// Save the current binds under a name.
    pub fn save_binds(&mut self, name: &str) -> Result<(), crate::bind_profile::BindProfileError> {
        self.bind_profile.save(name, &self.binds)
    }

    /// Load binds from a named profile.
    pub fn load_binds(&mut self, name: &str) -> Result<(), crate::bind_profile::BindProfileError> {
        self.bind_profile.load(name, &mut self.binds)
    }

    /// Hover description for a key.
    pub fn hover(&self, key: &str) -> &'static str {
        crate::help::Help::hover(key)
    }

    /// Press a key and run the bound action. Returns the action name.
    /// Returns None if the signed-in role cannot do it.
    pub fn press(&mut self, key: &str) -> Option<String> {
        let action = self.binds.action(key)?.to_string();
        if !self.can(&action) {
            self.log.add(&format!("denied {key}: {action}"));
            return None;
        }
        self.log.add(&format!("press {key}: {action}"));
        self.key_macros.record(key);
        match action.as_str() {
            "record" => self.start_recording(),
            "stream" => { let _ = self.start_stream("rtmp://live"); }
            "vcam" => self.start_vcam(),
            "stop_record" => self.stop_recording(),
            "stop_stream" => self.stop_stream(),
            "stop_vcam" => self.stop_vcam(),
            _ => {}
        }
        Some(action)
    }

    pub fn macros(&self) -> &Macro {
        &self.key_macros
    }

    /// Start recording a macro.
    pub fn start_macro(&mut self, name: &str) -> Result<(), crate::macro_seq::MacroError> {
        self.key_macros.start(name)
    }

    /// Stop recording the macro.
    pub fn stop_macro(&mut self) {
        self.key_macros.stop();
    }

    /// Replay a macro. Returns how many keys ran.
    pub fn play_macro(&mut self, name: &str) -> usize {
        let seq = match self.key_macros.sequence(name) {
            Some(s) => s.to_vec(),
            None => return 0,
        };
        self.key_macros.stop();
        let mut n = 0;
        for key in seq {
            if self.press(&key).is_some() {
                n += 1;
            }
        }
        n
    }

    /// Is a macro currently recording.
    pub fn is_recording_macro(&self) -> bool {
        self.key_macros.is_recording()
    }

    /// Edit a macro's sequence.
    pub fn edit_macro(&mut self, name: &str, keys: &[&str]) -> Result<(), crate::macro_seq::MacroError> {
        self.key_macros.edit(name, keys)
    }

    /// Rename a macro.
    pub fn rename_macro(&mut self, from: &str, to: &str) -> Result<(), crate::macro_seq::MacroError> {
        self.key_macros.rename(from, to)
    }

    /// Cancel a macro.
    pub fn cancel_macro(&mut self, name: &str) -> bool {
        self.key_macros.cancel(name)
    }

    /// All macro names.
    pub fn macro_names(&self) -> Vec<&str> {
        self.key_macros.names()
    }

    pub fn schedule(&self) -> &Schedule {
        &self.timed
    }

    /// Schedule a macro at a frame.
    pub fn schedule_at(&mut self, name: &str, frame: u64) -> Result<(), crate::schedule::ScheduleError> {
        self.timed.at(name, frame)
    }

    /// Schedule a macro at a time and date.
    pub fn schedule_datetime(&mut self, name: &str, datetime: &str) -> Result<(), crate::schedule::ScheduleError> {
        self.timed.at_datetime(name, datetime)
    }

    /// Run macros due at this frame. Returns how many ran.
    pub fn run_due(&mut self, frame: u64) -> usize {
        let due = self.timed.due_at(frame);
        let names: Vec<String> = due.iter().map(|s| s.to_string()).collect();
        let mut n = 0;
        for name in names {
            n += self.play_macro(&name);
            self.timed.cancel(&name);
        }
        n
    }

    pub fn fingerprints(&self) -> &FingerprintStore {
        &self.fingerprints
    }

    pub fn cache(&self) -> &FileCache {
        &self.cache
    }

    /// Store a file's bytes and fingerprint.
    pub fn remember(&mut self, path: &str, bytes: &[u8]) -> Result<(), crate::fingerprint::FingerprintError> {
        self.cache.store(path, bytes).map_err(|_| crate::fingerprint::FingerprintError::EmptyPath)?;
        self.fingerprints.store(path, bytes)
    }

    /// Rebuild a missing file from the cache. Returns the bytes.
    pub fn rebuild(&self, path: &str) -> Result<&[u8], crate::file_cache::CacheError> {
        self.cache.rebuild(path)
    }

    pub fn org(&self) -> &Organization {
        &self.team
    }

    pub fn org_mut(&mut self) -> &mut Organization {
        &mut self.team
    }

    /// Invite a member into the organization.
    pub fn invite(&mut self, name: &str, role: &str) -> Result<(), crate::invite::InviteError> {
        let invite = crate::invite::Invite::new(name, role)?;
        invite.accept(&mut self.team).map_err(|_| crate::invite::InviteError::AlreadyMember)
    }

    pub fn signed_in(&self) -> &SignIn {
        &self.signed_in
    }

    /// Sign in as a member.
    pub fn sign_in(&mut self, name: &str, role: &str) -> Result<(), crate::sign_in::SessionError> {
        self.signed_in.sign_in(name, role)
    }

    /// Can the signed-in member do this action.
    pub fn can(&self, action: &str) -> bool {
        match self.signed_in.role() {
            Some(role) => crate::permission::Permission::can(role, action),
            None => false,
        }
    }

    pub fn log(&self) -> &Log {
        &self.log
    }

    /// Undo the last action. Returns the reverse action that ran.
    pub fn undo(&mut self) -> Option<String> {
        let last = self.log.last()?.to_string();
        let action = last.split(": ").nth(1)?;
        let reverse = crate::undo::Undo::reverse(action)?.to_string();
        match reverse.as_str() {
            "record" => self.start_recording(),
            "stream" => { let _ = self.start_stream("rtmp://live"); }
            "vcam" => self.start_vcam(),
            "stop_record" => self.stop_recording(),
            "stop_stream" => self.stop_stream(),
            "stop_vcam" => self.stop_vcam(),
            _ => {}
        }
        self.log.add(&format!("undo: {reverse}"));
        Some(reverse)
    }

    /// Redo the last undone action. Returns the action that ran.
    pub fn redo(&mut self) -> Option<String> {
        let action = crate::redo::Redo::next(&self.log)?;
        match action.as_str() {
            "record" => self.start_recording(),
            "stream" => { let _ = self.start_stream("rtmp://live"); }
            "vcam" => self.start_vcam(),
            "stop_record" => self.stop_recording(),
            "stop_stream" => self.stop_stream(),
            "stop_vcam" => self.stop_vcam(),
            _ => {}
        }
        self.log.add(&format!("redo: {action}"));
        Some(action)
    }

    /// Start streaming the program to a target.
    pub fn start_stream(&mut self, target: &str) -> Result<(), crate::program_stream::StreamError> {
        self.stream.start(target)
    }

    /// Stop streaming the program.
    pub fn stop_stream(&mut self) {
        self.stream.stop();
    }

    /// Start recording the program.
    pub fn start_recording(&mut self) {
        self.record.start();
    }

    /// Stop recording the program.
    pub fn stop_recording(&mut self) {
        self.record.stop();
    }

    /// Start a fade from preview to program.
    pub fn fade_to_program(&mut self) {
        self.studio.fade();
    }

    /// Advance the fade. Returns true when complete.
    pub fn fade_step(&mut self, dt: f32) -> bool {
        self.studio.fade_step(dt)
    }

    /// Switch the live camera and mark it on the multiview.
    pub fn switch_camera(&mut self, input: &crate::input_device::InputDevice, cameras: &[&str]) -> Option<String> {
        let switched = self.feed.from_key(input, cameras);
        if let Some(ref cam) = switched {
            let _ = self.multiview.set_live(cam);
            let _ = self.tally.set_live(cam);
        }
        switched
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
