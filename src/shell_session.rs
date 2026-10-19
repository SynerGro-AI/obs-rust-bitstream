//! Safe pure ShellSession (save and load state).
//!
//! Saves the active tool, option, and layer count.
//! Loads them back. Round-trips. No unwraps.
//!
//! Differential: a different state produces a different save.

use crate::active_tool::ToolSource;
use crate::workspace::Workspace;

#[derive(Clone, Debug, PartialEq)]
pub struct SessionState {
    pub tool: String,
    pub size: f32,
    pub layers: usize,
}

pub struct ShellSession;

impl ShellSession {
    /// Save the workspace and layer count.
    pub fn save(workspace: &Workspace, layers: usize) -> SessionState {
        SessionState {
            tool: workspace.active().to_string(),
            size: workspace.options().get("size").unwrap_or(0.0),
            layers,
        }
    }

    /// Load a state into a workspace. Returns the layer count.
    pub fn load(workspace: &mut Workspace, state: &SessionState) -> usize {
        workspace.click(&state.tool, ToolSource::Photoshop);
        workspace.options_mut().set("size", state.size);
        state.layers
    }

    /// Serialize to a line.
    pub fn to_line(state: &SessionState) -> String {
        format!("tool={} size={:.0} layers={}", state.tool, state.size, state.layers)
    }

    /// Parse a line. None if malformed.
    pub fn from_line(line: &str) -> Option<SessionState> {
        let mut tool = None;
        let mut size = None;
        let mut layers = None;
        for part in line.split_whitespace() {
            if let Some(v) = part.strip_prefix("tool=") {
                tool = Some(v.to_string());
            } else if let Some(v) = part.strip_prefix("size=") {
                size = v.parse().ok();
            } else if let Some(v) = part.strip_prefix("layers=") {
                layers = v.parse().ok();
            }
        }
        Some(SessionState {
            tool: tool?,
            size: size?,
            layers: layers?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut w = Workspace::new(32, 32);
        w.click("Brush", ToolSource::Photoshop);
        w.options_mut().set("size", 30.0);
        let state = ShellSession::save(&w, 2);
        let line = ShellSession::to_line(&state);
        let loaded = ShellSession::from_line(&line).unwrap();
        let mut w2 = Workspace::new(32, 32);
        let layers = ShellSession::load(&mut w2, &loaded);
        assert_eq!(w2.active(), "Brush");
        assert!((w2.options().get("size").unwrap() - 30.0).abs() < 0.01);
        assert_eq!(layers, 2);
    }

    #[test]
    fn malformed_is_none() {
        assert!(ShellSession::from_line("nope").is_none());
    }

    #[test]
    fn differential() {
        let mut a = Workspace::new(16, 16);
        let mut b = Workspace::new(16, 16);
        a.click("Brush", ToolSource::Photoshop);
        b.click("Crop", ToolSource::Photoshop);
        assert_ne!(ShellSession::save(&a, 1), ShellSession::save(&b, 1));
    }
}
