//! The App side of `.seraphina` projects: building and applying the
//! manifest, save / open / new with unsaved-change prompts, the dirty
//! indicator, autosave and crash recovery.

use crate::animate::AnimFile;
use crate::assistant_panel::ChatFile;
use crate::composite::CompFile;
use crate::materials::MaterialsFile;
use crate::output::OutputConfig;
use crate::paint::PaintFile;
use crate::project::{self, Assets, Container, Header};
use crate::scene::SceneFile;
use crate::stream::StreamFile;
use crate::theme;
use crate::ToastKind;
use crate::{App, History, Mode};
use eframe::egui;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

/// Window layout saved with a project (theme and panels).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiFile {
    pub mode: String,
    pub dark: bool,
    pub show_tools: bool,
    pub show_inspector: bool,
    pub show_timeline: bool,
    pub show_shelf: bool,
    pub show_assistant: bool,
    pub dock_tab: usize,
    pub shelf_tab: usize,
}

/// `manifest.json`. Every section is optional so a project written by a
/// build without some mode still opens.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub header: Header,
    /// Autosaves remember which project file the work belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint: Option<PaintFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anim: Option<AnimFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp: Option<CompFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<SceneFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materials: Option<MaterialsFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<StreamFile>,
    /// Output / streaming settings with the stream key and URL
    /// credentials removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat: Option<ChatFile>,
}

/// Work queued by a prompt button, run after the modal closes.
type Deferred = Box<dyn FnOnce(&mut App)>;

/// What to do once an unsaved-changes prompt is answered.
#[derive(Clone, Debug, PartialEq)]
pub enum After {
    Nothing,
    Open(Option<PathBuf>),
    New,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Prompt {
    /// The last session crashed; its autosave can be recovered.
    Recover,
    /// "Save changes to X?" before `After`.
    Unsaved(After),
    /// Type a path to save to (in-app dialog).
    SavePath(After),
    /// Type a path to open (in-app dialog).
    OpenPath,
    /// The chosen save path exists.
    Overwrite(PathBuf, After),
}

pub struct ProjectState {
    /// The file this workspace was last saved to or opened from.
    pub path: Option<PathBuf>,
    /// Bumped whenever document state changes outside the paint and
    /// animate edit counters (detected by `content_hash`).
    pub edits: u64,
    saved_stamp: u64,
    last_hash: u64,
    hash_at: Option<Instant>,
    /// Save the Seraphina conversation in projects.
    pub include_chat: bool,
    pub session: project::Session,
    pub autosave_every: Duration,
    autosave_at: Instant,
    autosave_stamp: u64,
    autosave_job: Option<Receiver<Result<PathBuf, String>>>,
    pub prompt: Option<Prompt>,
    pub path_text: String,
    /// `--allow-file-commands`: scripts may save and open projects.
    pub script_files: bool,
    allow_close: bool,
    title: String,
}

impl ProjectState {
    pub fn new(session: project::Session) -> Self {
        let every = std::env::var("SERAPHINA_AUTOSAVE_SECS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|s| *s >= 1.0)
            .unwrap_or(project::AUTOSAVE_SECS);
        let prompt = session.recovery.as_ref().map(|_| Prompt::Recover);
        Self {
            path: None,
            edits: 0,
            saved_stamp: 0,
            last_hash: 0,
            hash_at: None,
            include_chat: true,
            session,
            autosave_every: Duration::from_secs_f64(every),
            autosave_at: Instant::now(),
            autosave_stamp: 0,
            autosave_job: None,
            prompt,
            path_text: String::new(),
            script_files: false,
            allow_close: false,
            title: String::new(),
        }
    }
}

/// Fields that are view state, not document state: changing them does not
/// mark the project dirty (they are still saved).
const VIEW_KEYS: &[&str] = &[
    "clock",
    "search",
    "selected",
    "prev_selected",
    "yaw",
    "pitch",
    "dist",
    "grid",
    "gizmo",
    "shading",
    "shelf_tab",
    "builder_tab",
    "atom_note",
];

fn hash_sections(sections: &[Value]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for s in sections {
        let mut s = s.clone();
        if let Some(o) = s.as_object_mut() {
            for k in VIEW_KEYS {
                o.remove(*k);
            }
        }
        s.to_string().hash(&mut h);
    }
    h.finish()
}

impl App {
    /// Changes everything: any edit in any mode moves it.
    pub fn edit_stamp(&self) -> u64 {
        self.paint.edits + self.anim.edits + self.project.edits
    }

    pub fn is_dirty(&self) -> bool {
        self.edit_stamp() != self.project.saved_stamp
    }

    /// Up-to-date dirty check: re-hashes now instead of waiting for the
    /// next periodic scan. Used before anything that could drop work.
    pub fn check_dirty(&mut self) -> bool {
        self.project.hash_at = None;
        self.track_changes();
        self.is_dirty()
    }

    pub(crate) fn mark_saved(&mut self) {
        self.project.last_hash = self.content_hash();
        self.project.saved_stamp = self.edit_stamp();
    }

    /// Mark the current state as unsaved (after a recovery).
    fn mark_dirty(&mut self) {
        self.project.last_hash = self.content_hash();
        self.project.saved_stamp = u64::MAX;
    }

    /// Hash of the non-raster document state (3D, materials, composite
    /// parameters, stream, output settings). Paint and Animate count their
    /// own edits.
    fn content_hash(&self) -> u64 {
        let mut scratch = Assets::default();
        let to = |v: Result<Value, serde_json::Error>| v.unwrap_or(Value::Null);
        hash_sections(&[
            to(serde_json::to_value(self.scene.to_file())),
            to(serde_json::to_value(self.materials.to_file())),
            to(serde_json::to_value(self.comp.to_file(&mut scratch))),
            to(serde_json::to_value(self.stream.to_file(&mut scratch))),
            to(serde_json::to_value(self.output.project_cfg())),
        ])
    }

    /// Notice edits that did not go through an edit counter.
    fn track_changes(&mut self) {
        let due = self
            .project
            .hash_at
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(400));
        if !due {
            return;
        }
        self.project.hash_at = Some(Instant::now());
        let h = self.content_hash();
        if h != self.project.last_hash {
            self.project.edits += 1;
            self.project.last_hash = h;
        }
    }

    /// Project name for the title bar.
    pub fn project_name(&self) -> String {
        self.project
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// "name ●" when there are unsaved changes.
    pub fn title_label(&self) -> String {
        if self.is_dirty() {
            format!("{} ●", self.project_name())
        } else {
            self.project_name()
        }
    }

    // ---- manifest ---------------------------------------------------------

    pub fn to_container(&self, include_chat: bool, source: Option<&Path>) -> Container {
        let mut assets = Assets::default();
        let m = Manifest {
            header: project::header(),
            source_path: source.map(|p| p.to_string_lossy().into_owned()),
            ui: Some(UiFile {
                mode: self.mode.label().into(),
                dark: self.dark,
                show_tools: self.show_tools,
                show_inspector: self.show_inspector,
                show_timeline: self.show_timeline,
                show_shelf: self.show_shelf,
                show_assistant: self.show_assistant,
                dock_tab: self.dock_tab,
                shelf_tab: self.shelf_tab,
            }),
            paint: Some(self.paint.to_file(&mut assets)),
            anim: Some(self.anim.to_file(&mut assets)),
            comp: Some(self.comp.to_file(&mut assets)),
            scene: Some(self.scene.to_file()),
            materials: Some(self.materials.to_file()),
            stream: Some(self.stream.to_file(&mut assets)),
            output: Some(self.output.project_cfg()),
            chat: include_chat.then(|| self.assistant.to_file()),
        };
        Container {
            manifest: serde_json::to_value(&m).unwrap_or(Value::Null),
            assets,
        }
    }

    /// Replace the workspace with a project. All-or-nothing: if any section
    /// fails, the previous workspace is put back.
    pub fn apply_container(&mut self, c: &Container) -> Result<Manifest, String> {
        let m: Manifest = serde_json::from_value(c.manifest.clone())
            .map_err(|e| format!("manifest.json: {}", e))?;
        let backup = self.to_container(true, None);
        let backup_mode = self.mode;
        match self.apply_manifest(m.clone(), &c.assets) {
            Ok(()) => Ok(m),
            Err(e) => {
                if let Ok(b) = serde_json::from_value::<Manifest>(backup.manifest) {
                    let _ = self.apply_manifest(b, &backup.assets);
                }
                self.mode = backup_mode;
                Err(e)
            }
        }
    }

    fn apply_manifest(&mut self, m: Manifest, a: &Assets) -> Result<(), String> {
        if let Some(f) = m.paint {
            self.paint
                .load_file(f, a)
                .map_err(|e| format!("Paint: {}", e))?;
        }
        if let Some(f) = m.anim {
            self.anim
                .load_file(f, a)
                .map_err(|e| format!("Animate: {}", e))?;
        }
        if let Some(f) = m.comp {
            self.comp
                .load_file(f, a)
                .map_err(|e| format!("Composite: {}", e))?;
        }
        if let Some(f) = m.scene {
            self.scene.load_file(f).map_err(|e| format!("3D: {}", e))?;
        }
        if let Some(f) = m.materials {
            self.materials
                .load_file(f)
                .map_err(|e| format!("Materials: {}", e))?;
        }
        if let Some(f) = m.stream {
            self.stream
                .load_file(f, a)
                .map_err(|e| format!("Stream: {}", e))?;
        }
        if let Some(c) = m.output {
            self.output.load_project_cfg(c);
        }
        if let Some(c) = m.chat {
            self.assistant.load_file(c);
        }
        if let Some(u) = m.ui {
            if let Some(mode) = Mode::parse(&u.mode) {
                self.mode = mode;
            }
            if u.dark != self.dark {
                self.set_dark(u.dark);
            }
            self.show_tools = u.show_tools;
            self.show_inspector = u.show_inspector;
            self.show_timeline = u.show_timeline;
            self.show_shelf = u.show_shelf;
            self.show_assistant = u.show_assistant;
            self.dock_tab = u.dock_tab.min(1);
            self.shelf_tab = u.shelf_tab;
        }
        // undo history and AI batch snapshots belong to the old workspace
        self.scene_hist = History::new();
        self.comp_hist = History::new();
        self.mat_hist = History::new();
        self.snapshots.clear();
        Ok(())
    }

    // ---- save / open / new ------------------------------------------------

    /// Write the project to `path` (atomically).
    pub fn save_project_to(&mut self, path: &Path) -> Result<String, String> {
        if let Some(d) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {}", d.display(), e))?;
        }
        let c = self.to_container(self.project.include_chat, None);
        project::write(path, &c)?;
        self.project.path = Some(path.to_path_buf());
        self.mark_saved();
        // the saved file supersedes the autosave
        if let Some(a) = self.project.session.autosave_path() {
            let _ = std::fs::remove_file(a);
        }
        self.project.autosave_stamp = self.edit_stamp();
        let msg = format!("Saved {}", path.display());
        self.toasts.push(ToastKind::Success, &msg);
        Ok(msg)
    }

    pub fn open_project_from(&mut self, path: &Path) -> Result<String, String> {
        let c = project::read(path)?;
        self.apply_container(&c)?;
        self.project.path = Some(path.to_path_buf());
        self.mark_saved();
        let msg = format!("Opened {}", path.display());
        self.toasts.push(ToastKind::Success, &msg);
        Ok(msg)
    }

    /// Start over with the launch workspace.
    pub fn new_project(&mut self) -> String {
        self.paint = crate::paint::PaintState::new(&mut self.toasts);
        self.anim = crate::animate::AnimState::new();
        self.comp = crate::composite::CompState::new();
        self.scene = crate::scene::SceneState::new();
        self.materials = crate::materials::MaterialsState::new();
        self.lab = crate::lab::LabState::new();
        self.stream = crate::stream::StreamState::new();
        self.scene_hist = History::new();
        self.comp_hist = History::new();
        self.mat_hist = History::new();
        self.snapshots.clear();
        self.project.path = None;
        self.project.edits += 1;
        self.mark_saved();
        "New project".into()
    }

    fn default_save_path(&self) -> PathBuf {
        match &self.project.path {
            Some(p) => p.clone(),
            None => project::documents_dir().join("Untitled.seraphina"),
        }
    }

    /// `save_project`: to the current file, or ask where.
    pub fn cmd_save_project(
        &mut self,
        path: Option<&str>,
        save_as: bool,
    ) -> Result<String, String> {
        match (path, &self.project.path) {
            (Some(p), _) => {
                let p = project::with_ext(p);
                self.save_project_to(&p)
            }
            (None, Some(p)) if !save_as => {
                let p = p.clone();
                self.save_project_to(&p)
            }
            _ => {
                self.ask_save_path(After::Nothing);
                Ok("choose where to save the project".into())
            }
        }
    }

    /// `open_project`: asks about unsaved changes first.
    pub fn cmd_open_project(&mut self, path: Option<&str>) -> Result<String, String> {
        let after = After::Open(path.map(project::with_ext));
        if self.check_dirty() {
            self.project.prompt = Some(Prompt::Unsaved(after));
            return Ok("unsaved changes: the user is asked to save them first".into());
        }
        self.run_after(after)
    }

    pub fn cmd_new_project(&mut self) -> Result<String, String> {
        if self.check_dirty() {
            self.project.prompt = Some(Prompt::Unsaved(After::New));
            return Ok("unsaved changes: the user is asked to save them first".into());
        }
        self.run_after(After::New)
    }

    pub(crate) fn ask_save_path(&mut self, after: After) {
        self.project.path_text = self.default_save_path().to_string_lossy().into_owned();
        self.project.prompt = Some(Prompt::SavePath(after));
    }

    fn ask_open_path(&mut self) {
        self.project.path_text = project::documents_dir().to_string_lossy().into_owned();
        self.project.prompt = Some(Prompt::OpenPath);
    }

    fn run_after(&mut self, after: After) -> Result<String, String> {
        match after {
            After::Nothing => Ok(String::new()),
            After::Open(Some(p)) => self.open_project_from(&p),
            After::Open(None) => {
                self.ask_open_path();
                Ok("choose a project to open".into())
            }
            After::New => Ok(self.new_project()),
            After::Quit => {
                self.project.allow_close = true;
                self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                Ok("closing".into())
            }
        }
    }

    /// Save to `path` (asking about an existing file unless it is the
    /// current project), then continue with `after`.
    fn save_then(&mut self, path: PathBuf, after: After, confirmed: bool) {
        if !confirmed && path.exists() && self.project.path.as_deref() != Some(path.as_path()) {
            self.project.prompt = Some(Prompt::Overwrite(path, after));
            return;
        }
        match self.save_project_to(&path) {
            Ok(_) => {
                if let Err(e) = self.run_after(after) {
                    self.toasts.push(ToastKind::Error, &e);
                }
            }
            Err(e) => self
                .toasts
                .push(ToastKind::Error, &format!("Save failed: {}", e)),
        }
    }

    // ---- autosave and recovery ----------------------------------------------

    /// Write the autosave now (synchronously). Returns the file written.
    pub fn autosave_now(&mut self) -> Option<Result<PathBuf, String>> {
        let path = self.project.session.autosave_path()?;
        let c = self.to_container(true, self.project.path.as_deref());
        self.project.autosave_stamp = self.edit_stamp();
        Some(project::write(&path, &c).map(|_| path))
    }

    fn autosave_tick(&mut self) {
        if let Some(rx) = &self.project.autosave_job {
            match rx.try_recv() {
                Ok(Err(e)) => {
                    self.project.autosave_job = None;
                    self.toasts
                        .push(ToastKind::Error, &format!("Autosave failed: {}", e));
                }
                Ok(Ok(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.project.autosave_job = None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
            }
        }
        // never overwrite a crashed session's autosave before the user decides
        if self.project.session.recovery.is_some() {
            return;
        }
        let Some(path) = self.project.session.autosave_path() else {
            return;
        };
        if self.project.autosave_at.elapsed() < self.project.autosave_every {
            return;
        }
        self.project.autosave_at = Instant::now();
        let dirty = self.check_dirty();
        let stamp = self.edit_stamp();
        if stamp == self.project.autosave_stamp || !dirty {
            return;
        }
        self.project.autosave_stamp = stamp;
        let c = self.to_container(true, self.project.path.as_deref());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(project::write(&path, &c).map(|_| path));
        });
        self.project.autosave_job = Some(rx);
    }

    /// Load the crashed session's autosave (left unsaved and dirty).
    pub fn recover(&mut self) -> Result<String, String> {
        let Some(path) = self.project.session.recovery.clone() else {
            return Err("nothing to recover".into());
        };
        let c = project::read(&path)?;
        let m = self.apply_container(&c)?;
        self.project.session.recovery = None;
        self.project.path = m.source_path.map(PathBuf::from);
        self.mark_dirty();
        // keep the recovered work on disk until the next autosave replaces it
        self.project.autosave_stamp = self.edit_stamp();
        let msg = "Recovered your unsaved work from the last session".to_string();
        self.toasts.push(ToastKind::Success, &msg);
        Ok(msg)
    }

    /// Clean shutdown: the autosave and session marker go away.
    pub fn end_session(&mut self) {
        if self.project.session.recovery.is_none() {
            self.project.session.end();
        }
    }

    // ---- per-frame --------------------------------------------------------

    pub(crate) fn project_frame(&mut self, ctx: &egui::Context) {
        self.track_changes();
        self.autosave_tick();
        if ctx.input(|i| i.viewport().close_requested())
            && !self.project.allow_close
            && (self.check_dirty() || self.project.session.recovery.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.project.prompt.is_none() {
                self.project.prompt = Some(if self.project.session.recovery.is_some() {
                    Prompt::Recover
                } else {
                    Prompt::Unsaved(After::Quit)
                });
            }
        }
        let title = format!("{} — Seraphina Studio", self.title_label());
        if title != self.project.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.project.title = title;
        }
        // keep the autosave timer running while idle
        ctx.request_repaint_after(Duration::from_secs(5));
    }

    pub(crate) fn project_modals(&mut self, ctx: &egui::Context) {
        let Some(prompt) = self.project.prompt.clone() else {
            return;
        };
        let p = self.pal;
        let mut close = false;
        let mut then: Option<Deferred> = None;
        let name = self.project_name();
        egui::Modal::new(egui::Id::new("project-prompt"))
            .frame(
                egui::Frame::NONE
                    .fill(p.panel)
                    .stroke(egui::Stroke::new(1.0_f32, p.line))
                    .corner_radius(egui::CornerRadius::same(theme::R_LG as u8))
                    .inner_margin(egui::Margin::same(theme::SP_LG as i8)),
            )
            .show(ctx, |ui| {
                ui.set_width(440.0);
                let heading = |ui: &mut egui::Ui, t: &str| {
                    ui.label(egui::RichText::new(t).size(16.0).strong().color(p.text));
                    ui.add_space(theme::SP_SM);
                };
                match prompt.clone() {
                    Prompt::Recover => {
                        heading(ui, "Recover unsaved work?");
                        let when = self
                            .project
                            .session
                            .recovery
                            .as_ref()
                            .and_then(|f| std::fs::metadata(f).ok())
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.elapsed().ok())
                            .map(|e| format!(" (autosaved {} ago)", fmt_ago(e)))
                            .unwrap_or_default();
                        ui.label(
                            egui::RichText::new(format!(
                                "Seraphina Studio did not shut down cleanly last time{}. Your work in every mode was autosaved and can be restored.",
                                when
                            ))
                            .color(p.muted),
                        );
                        ui.add_space(theme::SP_MD);
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(&p, "Recover")).clicked() {
                                then = Some(Box::new(|a: &mut App| {
                                    if let Err(e) = a.recover() {
                                        a.toasts.push(
                                            ToastKind::Error,
                                            &format!("Recovery failed: {}", e),
                                        );
                                    }
                                }));
                                close = true;
                            }
                            if ui.button("Discard").clicked() {
                                then = Some(Box::new(|a: &mut App| {
                                    a.project.session.discard_recovery();
                                }));
                                close = true;
                            }
                        });
                    }
                    Prompt::Unsaved(after) => {
                        heading(ui, &format!("Save changes to {}?", name));
                        ui.label(
                            egui::RichText::new("Your changes will be lost if you don't save them.")
                                .color(p.muted),
                        );
                        ui.add_space(theme::SP_MD);
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(&p, "Save")).clicked() {
                                let a2 = after.clone();
                                then = Some(Box::new(move |a: &mut App| match a.project.path.clone() {
                                    Some(path) => a.save_then(path, a2, true),
                                    None => a.ask_save_path(a2),
                                }));
                                close = true;
                            }
                            if ui.button("Don't save").clicked() {
                                let a2 = after.clone();
                                then = Some(Box::new(move |a: &mut App| {
                                    if let Err(e) = a.run_after(a2) {
                                        a.toasts.push(ToastKind::Error, &e);
                                    }
                                }));
                                close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close = true;
                            }
                        });
                    }
                    Prompt::SavePath(after) => {
                        heading(ui, "Save project");
                        ui.label(egui::RichText::new("File").color(p.muted));
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.project.path_text)
                                .desired_width(f32::INFINITY),
                        );
                        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.add_space(theme::SP_MD);
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(&p, "Save")).clicked() || enter {
                                let path = project::with_ext(self.project.path_text.trim());
                                let a2 = after.clone();
                                then = Some(Box::new(move |a: &mut App| a.save_then(path, a2, false)));
                                close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close = true;
                            }
                        });
                    }
                    Prompt::OpenPath => {
                        heading(ui, "Open project");
                        ui.label(egui::RichText::new("File").color(p.muted));
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.project.path_text)
                                .desired_width(f32::INFINITY),
                        );
                        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.add_space(theme::SP_MD);
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(&p, "Open")).clicked() || enter {
                                let path = PathBuf::from(self.project.path_text.trim());
                                then = Some(Box::new(move |a: &mut App| {
                                    if let Err(e) = a.open_project_from(&path) {
                                        a.toasts.push(ToastKind::Error, &e);
                                    }
                                }));
                                close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close = true;
                            }
                        });
                    }
                    Prompt::Overwrite(path, after) => {
                        let fname = path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        heading(ui, &format!("Replace {}?", fname));
                        ui.label(
                            egui::RichText::new(format!(
                                "{} already exists. Replacing it overwrites its contents.",
                                path.display()
                            ))
                            .color(p.muted),
                        );
                        ui.add_space(theme::SP_MD);
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(&p, "Replace")).clicked() {
                                let (p2, a2) = (path.clone(), after.clone());
                                then = Some(Box::new(move |a: &mut App| a.save_then(p2, a2, true)));
                                close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close = true;
                            }
                        });
                    }
                }
            });
        if close {
            self.project.prompt = None;
            if let Some(f) = then {
                f(self);
            }
        }
    }
}

fn fmt_ago(d: Duration) -> String {
    let s = d.as_secs();
    match s {
        0..=59 => format!("{} s", s),
        60..=3599 => format!("{} min", s / 60),
        3600..=86399 => format!("{} h", s / 3600),
        _ => format!("{} days", s / 86400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assistant_panel::{AssistantPanel, ChatItem, ItemKind};
    use crate::commands::{AppCommand, Origin};

    pub(crate) fn test_app(ctx: &egui::Context) -> App {
        let a = AssistantPanel::with_provider(None, crate::prefs::Prefs::ephemeral());
        let mut app = App::with(ctx, a, crate::videogen::VideoGenPanel::unconfigured());
        // never touch the user's saved output settings or stream key
        let dir = std::env::temp_dir().join(format!("seraphina_proj_rec_{}", std::process::id()));
        app.output = crate::output::Output::new(
            OutputConfig {
                record_dir: dir.to_string_lossy().into_owned(),
                width: 320,
                height: 180,
                ..Default::default()
            },
            String::new(),
        );
        app.mark_saved();
        app
    }

    fn run(app: &mut App, json: &str) {
        let c: AppCommand =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{}: {}", json, e));
        if let Err(e) = app.dispatch(&c, Origin::User) {
            panic!("{} failed: {}", json, e);
        }
    }

    fn edit_paint(app: &mut App) {
        run(app, r#"{"cmd":"switch_mode","mode":"paint"}"#);
        run(app, r#"{"cmd":"add_layer","name":"Saved strokes"}"#);
        run(app, r##"{"cmd":"set_brush","size":24,"color":"#3366ff"}"##);
        run(
            app,
            r#"{"cmd":"brush_stroke","points":[[40,40],[200,120],[320,90]]}"#,
        );
        run(app, r#"{"cmd":"set_layer_opacity","opacity":70}"#);
        run(app, r#"{"cmd":"add_mask"}"#);
        run(app, r#"{"cmd":"add_group","name":"Group A"}"#);
        run(app, r#"{"cmd":"select_rect","x":10,"y":10,"w":50,"h":40}"#);
        run(
            app,
            r##"{"cmd":"add_text","text":"Hello","x":60,"y":200,"size":32,"color":"#ff0000"}"##,
        );
    }

    fn edit_anim(app: &mut App) {
        run(app, r#"{"cmd":"switch_mode","mode":"animate"}"#);
        run(app, r#"{"cmd":"set_keyframe"}"#);
        run(app, r#"{"cmd":"set_keyframe"}"#);
        let f = &mut app.anim.frames[1];
        f.data[..20_000].copy_from_slice(&[200, 30, 90, 255].repeat(5000));
        app.anim.edits += 1;
        app.anim.fps = 8.0;
    }

    fn edit_comp(app: &mut App) {
        run(app, r#"{"cmd":"switch_mode","mode":"composite"}"#);
        run(app, r#"{"cmd":"set_layer_opacity","opacity":35}"#);
        run(app, r#"{"cmd":"set_layer_blend","mode":"screen"}"#);
        run(app, r#"{"cmd":"timeline_scrub","time":1.0}"#);
        run(app, r#"{"cmd":"set_keyframe"}"#);
    }

    fn edit_scene(app: &mut App) {
        run(app, r#"{"cmd":"switch_mode","mode":"3d"}"#);
        run(app, r#"{"cmd":"scene_add","kind":"cube"}"#);
        run(
            app,
            r#"{"cmd":"scene_transform","translate":[1.5,0.25,-2.0],"rotate_y":30,"scale":1.4}"#,
        );
        run(app, r#"{"cmd":"scene_extrude","distance":0.6}"#);
        run(app, r#"{"cmd":"material_assign","material":"gold"}"#);
        run(app, r#"{"cmd":"scene_shading","mode":"shaded_wire"}"#);
    }

    fn edit_materials(app: &mut App) {
        run(app, r#"{"cmd":"materials_add","element":"Fe","count":2}"#);
        run(app, r#"{"cmd":"materials_add","element":"O","count":3}"#);
        run(app, r#"{"cmd":"materials_build"}"#);
        run(app, r#"{"cmd":"materials_save","name":"My rust"}"#);
        run(app, r#"{"cmd":"materials_preset","preset":"copper"}"#);
    }

    fn edit_stream(app: &mut App) {
        run(app, r#"{"cmd":"switch_mode","mode":"stream"}"#);
        run(app, r#"{"cmd":"run_reader","width":5,"reads":7}"#);
        run(app, r#"{"cmd":"run_reader","width":3,"reads":2}"#);
    }

    fn edit_output(app: &mut App) {
        app.output.cfg.rtmp_url = "rtmp://live.example.com/app".into();
        app.output.cfg.fps = 24;
        app.output.cfg.video_kbps = 3200;
    }

    fn edit_chat(app: &mut App) {
        app.assistant.items.push(ChatItem {
            kind: ItemKind::User,
            text: "Make the sky bluer".into(),
            rated: None,
        });
        app.assistant.items.push(ChatItem {
            kind: ItemKind::Assistant {
                source: "Grok".into(),
            },
            text: "Done, I raised the blue levels.".into(),
            rated: Some(true),
        });
    }

    fn edit_all(app: &mut App) {
        edit_paint(app);
        edit_anim(app);
        edit_comp(app);
        edit_scene(app);
        edit_materials(app);
        edit_stream(app);
        edit_output(app);
        edit_chat(app);
    }

    /// The saved state, minus the timestamp.
    fn state(app: &App) -> Container {
        let mut c = app.to_container(true, None);
        c.manifest["saved_at"] = Value::from(0);
        c
    }

    fn assert_same(a: &App, b: &App) {
        let (x, y) = (state(a), state(b));
        for (k, v) in x.manifest.as_object().unwrap() {
            assert_eq!(Some(v), y.manifest.get(k), "section '{}' differs", k);
        }
        assert_eq!(x.manifest, y.manifest);
        assert_eq!(
            x.assets.files.keys().collect::<Vec<_>>(),
            y.assets.files.keys().collect::<Vec<_>>()
        );
        for (k, v) in &x.assets.files {
            assert!(y.assets.files[k] == *v, "asset {} differs", k);
        }
    }

    fn round_trip(tag: &str, edit: fn(&mut App)) -> (App, App) {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        edit(&mut a);
        let path = project::test_dir(tag).join("work.seraphina");
        a.save_project_to(&path).unwrap();
        assert!(!a.is_dirty());
        let ctx2 = egui::Context::default();
        let mut b = test_app(&ctx2);
        b.open_project_from(&path).unwrap();
        assert!(!b.is_dirty());
        assert_eq!(b.project.path.as_deref(), Some(path.as_path()));
        assert_same(&a, &b);
        // and the reopened workspace saves to the same bytes of state
        let again = path.with_file_name("again.seraphina");
        b.save_project_to(&again).unwrap();
        let (c1, c2) = (
            project::read(&path).unwrap(),
            project::read(&again).unwrap(),
        );
        assert_eq!(c1.assets, c2.assets);
        (a, b)
    }

    #[test]
    fn paint_round_trips_losslessly() {
        let (a, b) = round_trip("paint", edit_paint);
        assert_eq!(a.paint.doc.layers.len(), b.paint.doc.layers.len());
        assert!(b.paint.doc.layers.iter().any(|l| l.name == "Saved strokes"));
        for (x, y) in a.paint.doc.layers.iter().zip(&b.paint.doc.layers) {
            assert!(x.raster.data == y.raster.data, "layer {} pixels", x.name);
        }
        assert_eq!(b.mode, Mode::Paint);
    }

    #[test]
    fn animate_round_trips_losslessly() {
        let (a, b) = round_trip("anim", edit_anim);
        assert_eq!(a.anim.frames.len(), b.anim.frames.len());
        assert!(a.anim.frames.len() >= 3);
        assert!(a.anim.frames[1].data == b.anim.frames[1].data);
        assert_eq!(b.anim.fps, 8.0);
    }

    #[test]
    fn composite_round_trips_losslessly() {
        let (a, b) = round_trip("comp", edit_comp);
        let (x, y) = (
            &a.comp.layers[a.comp.selected],
            &b.comp.layers[b.comp.selected],
        );
        assert_eq!(x.layer.opacity, y.layer.opacity);
        assert_eq!(x.layer.blend, y.layer.blend);
        assert_eq!(x.opacity, y.opacity);
    }

    #[test]
    fn scene_round_trips_losslessly() {
        let (a, b) = round_trip("scene", edit_scene);
        assert_eq!(a.scene.objs.len(), b.scene.objs.len());
        assert_eq!(
            format!("{:?}", a.scene.assign),
            format!("{:?}", b.scene.assign)
        );
        assert_eq!(b.scene.selected, a.scene.selected);
    }

    #[test]
    fn materials_round_trip_losslessly() {
        let (_, b) = round_trip("mats", edit_materials);
        assert!(b.materials.lib.profiles.contains_key("My rust"));
    }

    #[test]
    fn stream_round_trips_losslessly() {
        let (a, b) = round_trip("stream", edit_stream);
        assert_eq!(a.stream.width, b.stream.width);
        assert_eq!(b.mode, Mode::Stream);
    }

    #[test]
    fn output_settings_round_trip_without_secrets() {
        let (_, b) = round_trip("output", edit_output);
        assert_eq!(b.output.cfg.fps, 24);
        assert_eq!(b.output.cfg.video_kbps, 3200);

        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        a.output.cfg.rtmp_url =
            "rtmp://bob:hunter2@live.example.com/app/sk_live_12345?token=abc".into();
        let _ = a.output.set_key("sk_live_12345", false);
        let json = state(&a).manifest.to_string();
        for secret in ["hunter2", "sk_live_12345", "token=abc", "bob:"] {
            assert!(!json.contains(secret), "manifest leaks {}", secret);
        }
        assert!(json.contains("rtmp://live.example.com/app"));
    }

    #[test]
    fn chat_round_trips_and_is_optional() {
        let (_, b) = round_trip("chat", edit_chat);
        assert!(b
            .assistant
            .items
            .iter()
            .any(|i| i.text == "Make the sky bluer" && matches!(i.kind, ItemKind::User)));
        assert_eq!(b.assistant.convo_len(), 2);

        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        a.project.include_chat = false;
        edit_chat(&mut a);
        let path = project::test_dir("nochat").join("p.seraphina");
        a.save_project_to(&path).unwrap();
        assert!(project::read(&path).unwrap().manifest.get("chat").is_none());
    }

    #[test]
    fn every_mode_round_trips_together() {
        let (a, b) = round_trip("all", edit_all);
        assert_eq!(a.mode, b.mode);
    }

    #[test]
    fn kill_mid_edit_then_relaunch_recovers_every_mode() {
        let dir = project::test_dir("crash");
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        a.start_session(dir.clone());
        assert!(
            a.project.prompt.is_none(),
            "fresh dir has nothing to recover"
        );
        edit_all(&mut a);
        a.autosave_now().unwrap().unwrap();
        let expected = state(&a);
        // the process dies: no clean shutdown, marker and autosave stay behind
        drop(a);
        assert!(dir.join(project::SESSION_MARKER).exists());

        let ctx2 = egui::Context::default();
        let mut b = test_app(&ctx2);
        b.start_session(dir.clone());
        assert_eq!(b.project.prompt, Some(Prompt::Recover));
        b.recover().unwrap();
        assert!(b.is_dirty(), "recovered work is unsaved");
        let got = state(&b);
        assert_eq!(expected.manifest, got.manifest);
        assert_eq!(expected.assets, got.assets);
        // a clean exit after recovering removes the autosave
        b.end_session();
        assert!(!dir.join(project::AUTOSAVE_FILE).exists());
        let c = test_app(&ctx2);
        let mut c = c;
        c.start_session(dir);
        assert!(c.project.prompt.is_none());
    }

    #[test]
    fn recovery_remembers_the_project_file() {
        let dir = project::test_dir("crash-path");
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        a.start_session(dir.clone());
        let file = dir.join("mine.seraphina");
        a.save_project_to(&file).unwrap();
        edit_scene(&mut a);
        a.autosave_now().unwrap().unwrap();
        drop(a);
        let mut b = test_app(&ctx);
        b.start_session(dir);
        b.recover().unwrap();
        assert_eq!(b.project.path.as_deref(), Some(file.as_path()));
        assert_eq!(b.title_label(), "mine.seraphina ●");
    }

    #[test]
    fn discarding_recovery_deletes_the_autosave() {
        let dir = project::test_dir("discard");
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        a.start_session(dir.clone());
        edit_scene(&mut a);
        a.autosave_now().unwrap().unwrap();
        drop(a);
        let mut b = test_app(&ctx);
        b.start_session(dir.clone());
        b.project.session.discard_recovery();
        assert!(!dir.join(project::AUTOSAVE_FILE).exists());
    }

    #[test]
    fn dirty_tracks_document_edits_not_view_changes() {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        assert!(!a.is_dirty());
        assert_eq!(a.title_label(), "Untitled");
        run(&mut a, r#"{"cmd":"switch_mode","mode":"3d"}"#);
        run(&mut a, r#"{"cmd":"zoom","percent":200}"#);
        run(&mut a, r#"{"cmd":"set_theme","dark":false}"#);
        a.project.hash_at = None;
        a.track_changes();
        assert!(!a.is_dirty(), "view changes are not edits");
        run(&mut a, r#"{"cmd":"scene_add","kind":"cylinder"}"#);
        a.project.hash_at = None;
        a.track_changes();
        assert!(a.is_dirty());
        assert_eq!(a.title_label(), "Untitled ●");
        let path = project::test_dir("dirty").join("d.seraphina");
        a.save_project_to(&path).unwrap();
        assert!(!a.is_dirty());
        assert_eq!(a.title_label(), "d.seraphina");
        run(&mut a, r#"{"cmd":"switch_mode","mode":"paint"}"#);
        run(&mut a, r#"{"cmd":"brush_stroke","points":[[5,5],[50,50]]}"#);
        assert!(a.is_dirty());
    }

    #[test]
    fn open_with_unsaved_changes_asks_first() {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        let path = project::test_dir("ask").join("x.seraphina");
        a.save_project_to(&path).unwrap();
        edit_scene(&mut a);
        let before = a.scene.objs.len();
        let r = a.cmd_open_project(Some(path.to_str().unwrap())).unwrap();
        assert!(r.contains("unsaved"), "{}", r);
        assert_eq!(
            a.project.prompt,
            Some(Prompt::Unsaved(After::Open(Some(path.clone()))))
        );
        assert_eq!(a.scene.objs.len(), before, "nothing replaced yet");
        // "Don't save" continues with the open
        a.project.prompt = None;
        a.run_after(After::Open(Some(path))).unwrap();
        assert_eq!(a.scene.objs.len(), before - 1);
        assert!(a.cmd_new_project().is_ok());
    }

    #[test]
    fn save_as_never_silently_overwrites() {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        let path = project::test_dir("overwrite").join("taken.seraphina");
        std::fs::write(&path, b"someone else's file").unwrap();
        a.save_then(path.clone(), After::Nothing, false);
        assert_eq!(
            a.project.prompt,
            Some(Prompt::Overwrite(path.clone(), After::Nothing))
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"someone else's file");
        a.project.prompt = None;
        a.save_then(path.clone(), After::Nothing, true);
        assert!(project::read(&path).is_ok());
        // no path yet: save asks where, defaulting to Documents/Seraphina
        let mut b = test_app(&ctx);
        b.cmd_save_project(None, false).unwrap();
        assert_eq!(b.project.prompt, Some(Prompt::SavePath(After::Nothing)));
        assert!(b.project.path_text.ends_with("Untitled.seraphina"));
    }

    #[test]
    fn a_bad_file_leaves_the_workspace_alone() {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        edit_scene(&mut a);
        let before = state(&a);
        let dir = project::test_dir("bad");
        let junk = dir.join("junk.seraphina");
        std::fs::write(&junk, b"PK\x03\x04 not really").unwrap();
        assert!(a.open_project_from(&junk).is_err());
        // a valid zip whose paint section points at a missing asset
        let mut c = a.to_container(false, None);
        c.assets.files.clear();
        let broken = dir.join("broken.seraphina");
        project::write(&broken, &c).unwrap();
        let e = a.open_project_from(&broken).unwrap_err();
        assert!(e.contains("missing asset"), "{}", e);
        assert_eq!(before.manifest, state(&a).manifest);
        assert_eq!(before.assets, state(&a).assets);
    }

    #[test]
    fn scripts_need_a_flag_for_project_files() {
        let ctx = egui::Context::default();
        let mut a = test_app(&ctx);
        let dir = project::test_dir("script");
        let out = dir.join("scripted.seraphina");
        let script = dir.join("s.jsonl");
        std::fs::write(
            &script,
            format!(
                "{{\"cmd\":\"scene_add\",\"kind\":\"cube\"}}\n{{\"cmd\":\"save_project\",\"path\":{}}}\n",
                serde_json::to_string(out.to_str().unwrap()).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(a.run_script(script.to_str().unwrap()), 1);
        assert!(!out.exists());
        a.project.script_files = true;
        assert_eq!(a.run_script(script.to_str().unwrap()), 2);
        assert!(out.exists());
    }
}
