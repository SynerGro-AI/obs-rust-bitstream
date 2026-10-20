//! Live preference learning, modelled on Seraphina.AGI's
//! `MercyLearningHeart`: every event nudges a weight with the golden
//! conjugate EMA (α = 0.382) and the bank is saved as JSON under
//! `SERAPHINA_HOME` (default `~/.seraphina`). No batch retraining ever
//! happens; each accept / undo / 👍 / 👎 / tool choice is one update.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Golden conjugate EMA factor used by Seraphina.AGI's MercyLearningHeart.
pub const ALPHA: f32 = 0.382;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MercyBank {
    /// EMA weights in 0..=1 (or raw parameter averages for `param.*`).
    pub weights: BTreeMap<String, f32>,
    /// Event counts.
    pub counts: BTreeMap<String, u32>,
    /// Total incremental updates applied.
    pub updates: u64,
}

pub fn seraphina_home() -> PathBuf {
    if let Ok(h) = std::env::var("SERAPHINA_HOME") {
        if !h.trim().is_empty() {
            return PathBuf::from(h);
        }
    }
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".seraphina")
}

pub fn default_path() -> PathBuf {
    if let Ok(p) = std::env::var("SERAPHINA_STUDIO_PREFS") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    seraphina_home().join("studio").join("mercy_bank.json")
}

pub struct Prefs {
    pub bank: MercyBank,
    pub path: Option<PathBuf>,
    pub last_error: Option<String>,
    pub last_update: Option<(String, f32)>,
}

impl Prefs {
    /// Load from disk (missing file = fresh bank).
    pub fn load(path: PathBuf) -> Self {
        let bank = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Self { bank, path: Some(path), last_error: None, last_update: None }
    }

    /// In-memory only (tests).
    pub fn ephemeral() -> Self {
        Self { bank: MercyBank::default(), path: None, last_error: None, last_update: None }
    }

    fn save(&mut self) {
        let Some(p) = &self.path else { return };
        let res = (|| -> std::io::Result<()> {
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d)?;
            }
            let tmp = p.with_extension("json.tmp");
            std::fs::write(&tmp, serde_json::to_string_pretty(&self.bank).unwrap_or_default())?;
            std::fs::rename(&tmp, p)
        })();
        self.last_error = res.err().map(|e| e.to_string());
    }

    /// One EMA step toward `value`. Returns the new weight.
    pub fn ema_update(&mut self, key: &str, value: f32) -> f32 {
        let old = self.bank.weights.get(key).copied().unwrap_or(value);
        let new = ALPHA * value + (1.0 - ALPHA) * old;
        self.bank.weights.insert(key.to_string(), new);
        *self.bank.counts.entry(key.to_string()).or_insert(0) += 1;
        self.bank.updates += 1;
        self.last_update = Some((key.to_string(), new));
        self.save();
        new
    }

    pub fn tool_used(&mut self, mode: &str, tool: &str) {
        self.ema_update(&format!("tool.{}.{}", mode.to_lowercase(), tool), 1.0);
    }

    /// Remember a parameter the user chose (EMA of raw values).
    pub fn param_chosen(&mut self, key: &str, value: f32) {
        self.ema_update(&format!("param.{}", key), value);
    }

    /// An AI action the user kept (true) or undid (false).
    pub fn ai_outcome(&mut self, command: &str, accepted: bool) {
        let v = if accepted { 1.0 } else { 0.0 };
        self.ema_update(&format!("ai.{}", command), v);
        self.ema_update("ai.acceptance", v);
    }

    pub fn feedback(&mut self, helpful: bool) {
        self.ema_update("reply.helpfulness", if helpful { 1.0 } else { 0.0 });
    }

    /// Short summary injected into Seraphina's system prompt.
    pub fn summary(&self) -> String {
        let b = &self.bank;
        if b.updates == 0 {
            return "No learned preferences yet.".into();
        }
        let mut tools: Vec<(&String, &u32)> = b.counts.iter().filter(|(k, _)| k.starts_with("tool.")).collect();
        tools.sort_by(|a, b| b.1.cmp(a.1));
        let tools: Vec<String> = tools.iter().take(4).map(|(k, n)| format!("{} ×{}", k.trim_start_matches("tool."), n)).collect();
        let mut parts = vec![format!("{} incremental updates", b.updates)];
        if !tools.is_empty() {
            parts.push(format!("most-used tools: {}", tools.join(", ")));
        }
        let params: Vec<String> = b.weights.iter().filter(|(k, _)| k.starts_with("param.")).map(|(k, v)| format!("{} ≈ {:.0}", k.trim_start_matches("param."), v)).collect();
        if !params.is_empty() {
            parts.push(format!("typical settings: {}", params.join(", ")));
        }
        if let Some(a) = b.weights.get("ai.acceptance") {
            parts.push(format!("AI action acceptance {:.0}%", a * 100.0));
        }
        let disliked: Vec<String> = b.weights.iter().filter(|(k, v)| k.starts_with("ai.") && *k != "ai.acceptance" && **v < 0.5).map(|(k, _)| k.trim_start_matches("ai.").to_string()).collect();
        if !disliked.is_empty() {
            parts.push(format!("often undone: {} (ask before using these)", disliked.join(", ")));
        }
        if let Some(h) = b.weights.get("reply.helpfulness") {
            parts.push(format!("reply helpfulness {:.0}%", h * 100.0));
        }
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ema_is_incremental_and_golden() {
        let mut p = Prefs::ephemeral();
        assert_eq!(p.ema_update("x", 1.0), 1.0);
        let v = p.ema_update("x", 0.0);
        assert!((v - (1.0 - ALPHA)).abs() < 1e-6);
        assert_eq!(p.bank.updates, 2);
        assert_eq!(p.bank.counts["x"], 2);
    }

    #[test]
    fn outcomes_shape_the_summary() {
        let mut p = Prefs::ephemeral();
        assert!(p.summary().contains("No learned"));
        p.tool_used("paint", "brush");
        p.tool_used("paint", "brush");
        p.param_chosen("brush.size", 30.0);
        p.ai_outcome("set_layer_blend", false);
        p.ai_outcome("set_layer_opacity", true);
        p.feedback(true);
        let s = p.summary();
        assert!(s.contains("paint.brush ×2"), "{}", s);
        assert!(s.contains("brush.size ≈ 30"), "{}", s);
        assert!(s.contains("often undone: set_layer_blend"), "{}", s);
        assert!(s.contains("acceptance"), "{}", s);
    }

    #[test]
    fn persists_to_json() {
        let dir = std::env::temp_dir().join(format!("seraphina-prefs-{}", std::process::id()));
        let path = dir.join("bank.json");
        let _ = std::fs::remove_file(&path);
        {
            let mut p = Prefs::load(path.clone());
            p.feedback(true);
            assert!(p.last_error.is_none(), "{:?}", p.last_error);
        }
        let p = Prefs::load(path.clone());
        assert_eq!(p.bank.updates, 1);
        assert_eq!(p.bank.weights.get("reply.helpfulness"), Some(&1.0));
        let _ = std::fs::remove_dir_all(dir);
    }
}
