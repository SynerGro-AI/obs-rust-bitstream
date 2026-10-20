//! Grok Imagine video panel over a `VideoGenProvider`.
//!
//! `GrokImagine` follows the documented xAI REST flow: POST
//! `/v1/videos/generations` (or `/v1/videos/extensions`) returns a
//! `request_id`; GET `/v1/videos/{id}` until `status` is `done` (with
//! `video.url`) or `failed`. Finished clips are downloaded to
//! `~/.seraphina/studio/videos/`. Stitching writes an ffmpeg concat list
//! and runs ffmpeg when it is on PATH. `generate_image` covers
//! POST `/v1/images/generations` for Paint's "Generate layer".

use crate::theme::{self, Palette, SP_SM};
use crate::commands::AppCommand;
use eframe::egui;
use egui_phosphor::regular as ic;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

#[derive(Clone, Debug, PartialEq)]
pub enum JobStatus {
    Pending,
    Done { url: String, duration: f32 },
    Failed(String),
}

pub trait VideoGenProvider: Send + Sync {
    fn name(&self) -> String;
    fn start_generate(&self, prompt: &str, duration: u32) -> Result<String, String>;
    fn start_extend(&self, video_url: &str, prompt: &str) -> Result<String, String>;
    fn poll(&self, request_id: &str) -> Result<JobStatus, String>;
}

pub struct GrokImagine {
    base: String,
    key: String,
    pub model: String,
    pub resolution: String,
    pub image_model: String,
}

/// Minimal standard base64 decoder (no extra dependency).
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.split(',').next_back()?.trim();
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a' + 26) as u32,
            b'0'..=b'9' => (c - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        })
    };
    let bytes: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut n = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            n |= val(c)? << (18 - 6 * i);
        }
        let k = chunk.len();
        if k < 2 {
            return None;
        }
        out.push((n >> 16) as u8);
        if k > 2 {
            out.push((n >> 8) as u8);
        }
        if k > 3 {
            out.push(n as u8);
        }
    }
    Some(out)
}

impl GrokImagine {
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("SERAPHINA_GROK_API_KEY").or_else(|_| std::env::var("XAI_API_KEY")).ok().filter(|k| !k.trim().is_empty())?;
        let base = std::env::var("SERAPHINA_VIDEO_BASE_URL").unwrap_or_else(|_| "https://api.x.ai/v1".into());
        let model = std::env::var("SERAPHINA_VIDEO_MODEL").unwrap_or_else(|_| Self::DEFAULT_MODEL.into());
        let resolution = std::env::var("SERAPHINA_VIDEO_RESOLUTION").unwrap_or_else(|_| "480p".into());
        let image_model = std::env::var("SERAPHINA_IMAGE_MODEL").unwrap_or_else(|_| Self::DEFAULT_IMAGE_MODEL.into());
        Some(Self { base: base.trim_end_matches('/').to_string(), key: key.trim().to_string(), model, resolution, image_model })
    }

    pub const DEFAULT_MODEL: &'static str = "grok-imagine-video-1.5";
    pub const DEFAULT_IMAGE_MODEL: &'static str = "grok-imagine-image-2.0";

    /// Text-to-image; returns encoded image bytes (PNG/JPEG).
    pub fn generate_image(&self, prompt: &str) -> Result<Vec<u8>, String> {
        let v = self.post("/images/generations", json!({ "model": self.image_model, "prompt": prompt, "n": 1, "response_format": "b64_json" }))?;
        let d = v.pointer("/data/0").ok_or_else(|| format!("no image in response: {}", v.to_string().chars().take(200).collect::<String>()))?;
        if let Some(b) = d.get("b64_json").and_then(|b| b.as_str()) {
            return b64_decode(b).ok_or_else(|| "bad base64 image".to_string());
        }
        let url = d.get("url").and_then(|u| u.as_str()).ok_or("image response had neither b64_json nor url")?;
        let r = ureq::get(url).call().map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut r.into_reader(), &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        ureq::post(&format!("{}{}", self.base, path))
            .set("Authorization", &format!("Bearer {}", self.key))
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())
            .map_err(http_err)?
            .into_json()
            .map_err(|e| e.to_string())
    }
}

fn http_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, r) => {
            let body = r.into_string().unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&body).ok().and_then(|v| v.pointer("/error/message").or_else(|| v.get("error")).and_then(|m| m.as_str().map(String::from))).unwrap_or(body);
            format!("HTTP {}: {}", code, msg.chars().take(240).collect::<String>())
        }
        other => other.to_string(),
    }
}

pub fn parse_status(v: &Value) -> JobStatus {
    let err = || v.pointer("/error/message").and_then(|m| m.as_str()).map(String::from);
    match v.get("status").and_then(|s| s.as_str()).unwrap_or("pending") {
        "done" => match v.pointer("/video/url").and_then(|u| u.as_str()).filter(|u| !u.is_empty()) {
            Some(u) => JobStatus::Done { url: u.to_string(), duration: v.pointer("/video/duration").and_then(|d| d.as_f64()).unwrap_or(0.0) as f32 },
            None if v.pointer("/video/respect_moderation").and_then(|m| m.as_bool()) == Some(false) => JobStatus::Failed("blocked by content moderation".into()),
            None => JobStatus::Failed("done without a video URL".into()),
        },
        "failed" => JobStatus::Failed(err().unwrap_or_else(|| "generation failed".into())),
        "expired" => JobStatus::Failed("request expired".into()),
        _ => JobStatus::Pending,
    }
}

impl VideoGenProvider for GrokImagine {
    fn name(&self) -> String {
        format!("Grok Imagine · {} · {}", self.model, self.resolution)
    }

    fn start_generate(&self, prompt: &str, duration: u32) -> Result<String, String> {
        let v = self.post("/videos/generations", json!({ "model": self.model, "prompt": prompt, "duration": duration.clamp(1, 15), "aspect_ratio": "16:9", "resolution": self.resolution }))?;
        v.get("request_id").and_then(|r| r.as_str()).map(String::from).ok_or_else(|| format!("no request_id in {}", v))
    }

    fn start_extend(&self, video_url: &str, prompt: &str) -> Result<String, String> {
        let v = self.post("/videos/extensions", json!({ "model": self.model, "prompt": prompt, "video": { "url": video_url }, "duration": 6 }))?;
        v.get("request_id").and_then(|r| r.as_str()).map(String::from).ok_or_else(|| format!("no request_id in {}", v))
    }

    fn poll(&self, id: &str) -> Result<JobStatus, String> {
        let v: Value = ureq::get(&format!("{}/videos/{}", self.base, id)).set("Authorization", &format!("Bearer {}", self.key)).call().map_err(http_err)?.into_json().map_err(|e| e.to_string())?;
        Ok(parse_status(&v))
    }
}

pub struct Clip {
    pub prompt: String,
    pub status: JobStatus,
    pub local: Option<PathBuf>,
    pub selected: bool,
    rx: Option<Receiver<(JobStatus, Option<PathBuf>)>>,
}

pub struct VideoGenPanel {
    pub provider: Option<std::sync::Arc<dyn VideoGenProvider>>,
    pub prompt: String,
    pub duration: u32,
    pub clips: Vec<Clip>,
    pub extend_prompt: String,
    pub log: Vec<String>,
    pub imager: Option<std::sync::Arc<GrokImagine>>,
    image_rx: Option<Receiver<Result<(String, Vec<u8>), String>>>,
}

pub fn videos_dir() -> PathBuf {
    crate::prefs::seraphina_home().join("studio").join("videos")
}

impl VideoGenPanel {
    pub fn new() -> Self {
        let imager = GrokImagine::from_env().map(std::sync::Arc::new);
        let provider = imager.clone().map(|g| g as std::sync::Arc<dyn VideoGenProvider>);
        Self { provider, prompt: String::new(), duration: 6, clips: Vec::new(), extend_prompt: String::new(), log: Vec::new(), imager, image_rx: None }
    }

    pub fn unconfigured() -> Self {
        Self { provider: None, prompt: String::new(), duration: 6, clips: Vec::new(), extend_prompt: String::new(), log: Vec::new(), imager: None, image_rx: None }
    }

    pub fn image_busy(&self) -> bool {
        self.image_rx.is_some()
    }

    /// Start a text-to-image job; the result arrives via `take_image`.
    pub fn generate_image(&mut self, prompt: &str) -> Result<String, String> {
        let g = self.imager.clone().ok_or("Grok Imagine isn't configured: set XAI_API_KEY and restart")?;
        if prompt.trim().is_empty() {
            return Err("empty prompt".into());
        }
        if self.image_rx.is_some() {
            return Err("an image is already generating".into());
        }
        let (tx, rx) = channel();
        let pr = prompt.trim().to_string();
        let p2 = pr.clone();
        std::thread::spawn(move || {
            let _ = tx.send(g.generate_image(&p2).map(|b| (p2, b)));
        });
        self.image_rx = Some(rx);
        Ok(format!("generating image “{}” with {}", pr.chars().take(60).collect::<String>(), self.imager.as_ref().map(|g| g.image_model.clone()).unwrap_or_default()))
    }

    pub fn take_image(&mut self) -> Option<Result<(String, Vec<u8>), String>> {
        let r = self.image_rx.as_ref()?.try_recv().ok()?;
        self.image_rx = None;
        Some(r)
    }

    fn spawn(&mut self, prompt: String, start: Box<dyn FnOnce(&dyn VideoGenProvider) -> Result<String, String> + Send>) -> Result<String, String> {
        let p = self.provider.clone().ok_or("Grok Imagine isn't configured: set XAI_API_KEY and restart")?;
        let (tx, rx) = channel();
        let idx = self.clips.len();
        std::thread::spawn(move || {
            let id = match start(p.as_ref()) {
                Ok(id) => id,
                Err(e) => {
                    let _ = tx.send((JobStatus::Failed(e), None));
                    return;
                }
            };
            for _ in 0..180 {
                std::thread::sleep(std::time::Duration::from_secs(5));
                match p.poll(&id) {
                    Ok(JobStatus::Pending) => continue,
                    Ok(JobStatus::Done { url, duration }) => {
                        let dir = videos_dir();
                        let path = dir.join(format!("clip_{}_{}.mp4", idx + 1, id.chars().take(8).collect::<String>()));
                        let saved = std::fs::create_dir_all(&dir).ok().and_then(|_| ureq::get(&url).call().ok()).and_then(|r| {
                            let mut f = std::fs::File::create(&path).ok()?;
                            std::io::copy(&mut r.into_reader(), &mut f).ok()?;
                            Some(path.clone())
                        });
                        let _ = tx.send((JobStatus::Done { url, duration }, saved));
                        return;
                    }
                    Ok(s) => {
                        let _ = tx.send((s, None));
                        return;
                    }
                    Err(e) => {
                        let _ = tx.send((JobStatus::Failed(e), None));
                        return;
                    }
                }
            }
            let _ = tx.send((JobStatus::Failed("timed out after 15 minutes".into()), None));
        });
        self.clips.push(Clip { prompt: prompt.clone(), status: JobStatus::Pending, local: None, selected: true, rx: Some(rx) });
        Ok(format!("queued clip {}: “{}”", idx + 1, prompt.chars().take(60).collect::<String>()))
    }

    pub fn generate(&mut self, prompt: &str, duration: Option<u32>) -> Result<String, String> {
        if prompt.trim().is_empty() {
            return Err("empty prompt".into());
        }
        let d = duration.unwrap_or(self.duration).clamp(1, 15);
        let pr = prompt.trim().to_string();
        let p2 = pr.clone();
        self.spawn(pr, Box::new(move |p| p.start_generate(&p2, d)))
    }

    pub fn extend(&mut self, clip: usize, prompt: &str) -> Result<String, String> {
        let url = match self.clips.get(clip.saturating_sub(1)).map(|c| &c.status) {
            Some(JobStatus::Done { url, .. }) => url.clone(),
            Some(_) => return Err(format!("clip {} isn't finished", clip)),
            None => return Err(format!("no clip {}", clip)),
        };
        let p2 = prompt.to_string();
        self.spawn(format!("extend #{}: {}", clip, prompt), Box::new(move |p| p.start_extend(&url, &p2)))
    }

    /// Write a concat list (and run ffmpeg if available). Clip numbers are 1-based.
    pub fn stitch(&mut self, clips: Option<Vec<usize>>) -> Result<String, String> {
        let picks: Vec<usize> = clips.unwrap_or_else(|| self.clips.iter().enumerate().filter(|(_, c)| c.selected).map(|(i, _)| i + 1).collect());
        let files: Vec<PathBuf> = picks.iter().filter_map(|i| self.clips.get(i.saturating_sub(1)).and_then(|c| c.local.clone())).collect();
        if files.len() < 2 {
            return Err("need at least 2 downloaded clips to stitch".into());
        }
        let dir = videos_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let list = dir.join("stitch_list.txt");
        let body: String = files.iter().map(|f| format!("file '{}'\n", f.display().to_string().replace('\'', "'\\''"))).collect();
        std::fs::write(&list, body).map_err(|e| e.to_string())?;
        let out = dir.join("stitched.mp4");
        let ran = std::process::Command::new("ffmpeg").args(["-y", "-f", "concat", "-safe", "0", "-i"]).arg(&list).args(["-c", "copy"]).arg(&out).output();
        match ran {
            Ok(o) if o.status.success() => Ok(format!("stitched {} clips → {}", files.len(), out.display())),
            _ => Ok(format!("wrote concat list {} (ffmpeg not found; run: ffmpeg -f concat -safe 0 -i stitch_list.txt -c copy out.mp4)", list.display())),
        }
    }

    pub fn tick(&mut self) -> bool {
        let mut busy = false;
        for c in &mut self.clips {
            if let Some(rx) = &c.rx {
                busy = true;
                if let Ok((s, local)) = rx.try_recv() {
                    c.status = s;
                    c.local = local;
                    c.rx = None;
                }
            }
        }
        busy
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(ic::FILM_SLATE).size(16.0).color(p.accent));
            ui.label(egui::RichText::new("Grok Imagine").size(14.0).strong().color(p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match &self.provider {
                Some(pr) => {
                    theme::pill(ui, p, p.success, "Configured").on_hover_text(pr.name());
                }
                None => {
                    theme::pill(ui, p, p.warn, "Not configured").on_hover_text("Set XAI_API_KEY (or SERAPHINA_GROK_API_KEY) and restart");
                }
            });
        });
        ui.label(theme::muted(p, "Text-to-video and text-to-image via the xAI Imagine API (billed per second / per image). Clips are saved to ~/.seraphina/studio/videos."));
        ui.add_space(SP_SM);
        ui.add(egui::TextEdit::multiline(&mut self.prompt).hint_text("Describe a shot: “slow aerial push over the golden valley at dusk”").desired_rows(3).desired_width(f32::INFINITY));
        ui.horizontal(|ui| {
            ui.label(theme::muted(p, "Duration"));
            ui.add(egui::DragValue::new(&mut self.duration).range(1..=15).suffix(" s"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let ok = self.provider.is_some() && !self.prompt.trim().is_empty();
                if ui.add_enabled(ok, theme::primary_button(p, "Generate video")).on_disabled_hover_text("Needs XAI_API_KEY and a prompt").clicked() {
                    bus.push(AppCommand::VideoGenerate { prompt: self.prompt.clone(), duration: Some(self.duration) });
                }
                let label = if self.image_rx.is_some() { "Imaging…" } else { "Image → layer" };
                if ui.add_enabled(ok && self.image_rx.is_none(), theme::toggle_button(p, label, false)).on_hover_text("Generate a still with Grok Imagine and add it to Paint as a new layer").clicked() {
                    bus.push(AppCommand::GenerateImage { prompt: self.prompt.clone() });
                }
            });
        });
        ui.add_space(SP_SM);
        ui.label(theme::section(p, &format!("Clips · {}", self.clips.len())));
        if self.clips.is_empty() {
            ui.label(theme::muted(p, "No clips yet."));
        }
        let mut open = None;
        for (i, c) in self.clips.iter_mut().enumerate() {
            theme::card(ui, p, "", |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut c.selected, "");
                    ui.label(egui::RichText::new(format!("#{}", i + 1)).strong().color(p.accent));
                    let (col, txt) = match &c.status {
                        JobStatus::Pending => (p.warn, "Generating…".to_string()),
                        JobStatus::Done { duration, .. } => (p.success, format!("Done · {:.0}s{}", duration, if c.local.is_some() { " · saved" } else { "" })),
                        JobStatus::Failed(e) => (p.danger, format!("Failed: {}", e.chars().take(60).collect::<String>())),
                    };
                    theme::pill(ui, p, col, &txt);
                });
                ui.add(egui::Label::new(theme::muted(p, &c.prompt)).wrap());
                if let JobStatus::Done { url, .. } = &c.status {
                    if ui.add(theme::toggle_button(p, "Open in browser", false)).clicked() {
                        open = Some(url.clone());
                    }
                }
            });
        }
        if let Some(u) = open {
            ui.ctx().open_url(egui::OpenUrl::new_tab(u));
        }
        let done: Vec<usize> = self.clips.iter().enumerate().filter(|(_, c)| matches!(c.status, JobStatus::Done { .. })).map(|(i, _)| i + 1).collect();
        if !done.is_empty() {
            ui.add_space(SP_SM);
            ui.add(egui::TextEdit::singleline(&mut self.extend_prompt).hint_text("Extension prompt for the last finished clip").desired_width(f32::INFINITY));
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.extend_prompt.trim().is_empty(), theme::toggle_button(p, &format!("Extend #{}", done[done.len() - 1]), false)).clicked() {
                    bus.push(AppCommand::VideoExtend { clip: done[done.len() - 1], prompt: self.extend_prompt.clone() });
                }
                if ui.add(theme::toggle_button(p, "Stitch selected", false)).clicked() {
                    bus.push(AppCommand::VideoStitch { clips: None });
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_parsing() {
        assert_eq!(parse_status(&json!({"status":"pending"})), JobStatus::Pending);
        assert_eq!(parse_status(&json!({"status":"done","video":{"url":"https://v/x.mp4","duration":8}})), JobStatus::Done { url: "https://v/x.mp4".into(), duration: 8.0 });
        assert!(matches!(parse_status(&json!({"status":"failed"})), JobStatus::Failed(_)));
        assert!(matches!(parse_status(&json!({"status":"done"})), JobStatus::Failed(_)));
        assert_eq!(parse_status(&json!({"status":"failed","error":{"code":"internal_error","message":"boom"}})), JobStatus::Failed("boom".into()));
        assert_eq!(parse_status(&json!({"status":"done","video":{"url":null,"respect_moderation":false}})), JobStatus::Failed("blocked by content moderation".into()));
        assert_eq!(parse_status(&json!({"status":"pending","progress":40})), JobStatus::Pending);
    }

    #[test]
    fn base64_roundtrip() {
        assert_eq!(b64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(b64_decode("data:image/png;base64,aGk=").unwrap(), b"hi");
        assert!(b64_decode("@@@@").is_none());
    }

    #[test]
    fn unconfigured_is_a_clear_error() {
        let mut v = VideoGenPanel::unconfigured();
        assert!(v.generate_image("a cat").unwrap_err().contains("XAI_API_KEY"));
        assert!(v.generate("a cat", None).unwrap_err().contains("XAI_API_KEY"));
        assert!(v.stitch(None).is_err());
        assert!(v.extend(1, "more").is_err());
    }

    /// Live, billed: run explicitly with `cargo test --features ui --bin ui -- --ignored live_imagine`.
    /// Writes into SERAPHINA_LIVE_OUT (default: temp dir).
    #[test]
    #[ignore]
    fn live_imagine_video_and_image() {
        let Some(g) = GrokImagine::from_env() else {
            eprintln!("SKIPPED live_imagine_video_and_image: no XAI_API_KEY");
            return;
        };
        let out = std::env::var("SERAPHINA_LIVE_OUT").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
        std::fs::create_dir_all(&out).unwrap();
        let t0 = std::time::Instant::now();
        let img = g.generate_image("A gold gem logo on a near-black background, minimal, crisp").expect("image");
        let im = image::load_from_memory(&img).expect("decodes");
        std::fs::write(out.join("live_grok_imagine_image.jpg"), &img).unwrap();
        eprintln!("LIVE image: {} bytes, {}x{}, {:.1}s", img.len(), im.width(), im.height(), t0.elapsed().as_secs_f32());
        let t0 = std::time::Instant::now();
        let id = g.start_generate("Slow golden light sweeping across misty mountain peaks at dusk", 1).expect("start");
        eprintln!("LIVE video request accepted ({} @ {})", g.model, g.resolution);
        let mut status = JobStatus::Pending;
        for _ in 0..120 {
            std::thread::sleep(std::time::Duration::from_secs(5));
            status = g.poll(&id).expect("poll");
            if status != JobStatus::Pending {
                break;
            }
        }
        match status {
            JobStatus::Done { url, duration } => {
                let r = ureq::get(&url).call().expect("download");
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut r.into_reader(), &mut buf).unwrap();
                std::fs::write(out.join("live_grok_imagine_clip.mp4"), &buf).unwrap();
                eprintln!("LIVE video: {:.0}s clip, {} bytes, {:.0}s wall", duration, buf.len(), t0.elapsed().as_secs_f32());
                assert!(buf.len() > 1000);
            }
            other => panic!("video did not finish: {:?}", other),
        }
    }
}
