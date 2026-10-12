//! AI/SI integration scaffold for automated assistance.
//!
//! Simple variable adjustment for API backend selection.
//! Supports Grok, Claude, or Seraphina with fully automated ability
//! (stubs ready for real client wiring).
//!
//! Change `BACKEND` or the config fields to switch providers.

/// Supported AI backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Grok,
    Claude,
    Seraphina,
}

/// Simple configuration. Adjust these variables to switch APIs.
pub struct AiConfig {
    pub backend: Backend,
    pub api_key_env: &'static str, // e.g. "GROK_API_KEY", "ANTHROPIC_API_KEY"
    pub base_url: &'static str,
    pub model: &'static str,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        // Default to Grok. Change backend (and matching env/url/model) to switch.
        Self {
            backend: Backend::Grok,
            api_key_env: "GROK_API_KEY",
            base_url: "https://api.x.ai/v1",
            model: "grok-4",
            temperature: 0.2,
            max_tokens: 4096,
        }
    }
}

impl AiConfig {
    /// Claude preset.
    pub fn claude() -> Self {
        Self {
            backend: Backend::Claude,
            api_key_env: "ANTHROPIC_API_KEY",
            base_url: "https://api.anthropic.com/v1",
            model: "claude-sonnet-4",
            temperature: 0.2,
            max_tokens: 4096,
        }
    }

    /// Seraphina preset (local or custom endpoint).
    pub fn seraphina() -> Self {
        Self {
            backend: Backend::Seraphina,
            api_key_env: "SERAPHINA_API_KEY",
            base_url: "http://localhost:8080/v1", // adjust as needed
            model: "seraphina",
            temperature: 0.1,
            max_tokens: 8192,
        }
    }
}

/// Automated assistant stub.
/// In a full wiring this would call the selected backend to help generate
/// or review ports, tests, or patches.
pub struct AiAssistant {
    pub config: AiConfig,
}

impl AiAssistant {
    pub fn new(config: AiConfig) -> Self {
        Self { config }
    }

    /// Placeholder for an automated prompt. Returns a description of what
    /// would be sent. Wire a real HTTP client (reqwest, etc.) later.
    pub fn assist(&self, prompt: &str) -> String {
        format!(
            "[AI/{}] model={} temp={} max_tokens={} | prompt: {}",
            match self.config.backend {
                Backend::Grok => "Grok",
                Backend::Claude => "Claude",
                Backend::Seraphina => "Seraphina",
            },
            self.config.model,
            self.config.temperature,
            self.config.max_tokens,
            prompt
        )
    }

    /// Example automated task: review a generated skeleton.
    pub fn review_skeleton(&self, rust_code: &str) -> String {
        self.assist(&format!(
            "Review this OBS-to-Rust skeleton for correctness, bug fixes, and test coverage:\n{}",
            rust_code
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_grok() {
        let cfg = AiConfig::default();
        assert_eq!(cfg.backend, Backend::Grok);
        assert_eq!(cfg.api_key_env, "GROK_API_KEY");
    }

    #[test]
    fn switch_backends() {
        assert_eq!(AiConfig::claude().backend, Backend::Claude);
        assert_eq!(AiConfig::seraphina().backend, Backend::Seraphina);
    }

    #[test]
    fn assist_stub() {
        let ai = AiAssistant::new(AiConfig::default());
        let out = ai.assist("generate tests for bitstream");
        assert!(out.contains("Grok"));
        assert!(out.contains("generate tests for bitstream"));
    }
}
