//! App settings that must be known before the window opens (M3): the
//! renderer choice (D2). Stored as `~/.seraphina/studio/settings.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RendererPref {
    /// wgpu (DirectX 12 / Vulkan) on Windows, OpenGL elsewhere; the other
    /// one is tried when the first can't start.
    #[default]
    Auto,
    Wgpu,
    Glow,
}

impl RendererPref {
    pub const ALL: [RendererPref; 3] = [Self::Auto, Self::Wgpu, Self::Glow];
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic",
            Self::Wgpu => "DirectX 12 / Vulkan (wgpu)",
            Self::Glow => "OpenGL (glow)",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "wgpu" | "dx12" | "vulkan" | "directx" => Some(Self::Wgpu),
            "glow" | "gl" | "opengl" => Some(Self::Glow),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub renderer: RendererPref,
    /// Set when Automatic had to fall back; Automatic then starts with it.
    pub auto_fallback: Option<RendererPref>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            renderer: RendererPref::Auto,
            auto_fallback: None,
        }
    }
}

pub fn path() -> PathBuf {
    crate::output::studio_dir().join("settings.json")
}

impl AppSettings {
    pub fn load() -> (Self, Option<String>) {
        crate::project::load_json(&path(), "settings file")
    }
    pub fn save(&self) -> Result<(), String> {
        let s = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        crate::project::write_atomic(&path(), s.as_bytes()).map_err(|e| e.to_string())
    }

    /// Renderers to try, in order. `env` is `SERAPHINA_RENDERER`.
    pub fn renderer_order(&self, env: Option<&str>, windows: bool) -> Vec<RendererPref> {
        let platform = if windows {
            [RendererPref::Wgpu, RendererPref::Glow]
        } else {
            [RendererPref::Glow, RendererPref::Wgpu]
        };
        let first = env
            .and_then(RendererPref::parse)
            .filter(|r| *r != RendererPref::Auto)
            .or(match self.renderer {
                RendererPref::Auto => self.auto_fallback,
                r => Some(r),
            })
            .unwrap_or(platform[0]);
        let mut v = vec![first];
        v.extend(platform.iter().copied().filter(|r| *r != first));
        v
    }
}

pub fn to_eframe(r: RendererPref) -> eframe::Renderer {
    match r {
        RendererPref::Glow => eframe::Renderer::Glow,
        _ => eframe::Renderer::Wgpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RendererPref::*;

    #[test]
    fn renderer_order_has_a_fallback() {
        let s = AppSettings::default();
        assert_eq!(s.renderer_order(None, true), vec![Wgpu, Glow]);
        assert_eq!(s.renderer_order(None, false), vec![Glow, Wgpu]);
        assert_eq!(s.renderer_order(Some("opengl"), true), vec![Glow, Wgpu]);
        assert_eq!(s.renderer_order(Some("bogus"), true), vec![Wgpu, Glow]);
        let fb = AppSettings {
            auto_fallback: Some(Glow),
            ..Default::default()
        };
        assert_eq!(fb.renderer_order(None, true), vec![Glow, Wgpu]);
        let forced = AppSettings {
            renderer: Wgpu,
            auto_fallback: Some(Glow),
        };
        assert_eq!(forced.renderer_order(None, false), vec![Wgpu, Glow]);
    }

    #[test]
    fn old_or_partial_files_load() {
        let s: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(s, AppSettings::default());
        let s: AppSettings = serde_json::from_str(r#"{"renderer":"glow"}"#).unwrap();
        assert_eq!(s.renderer, Glow);
    }
}
