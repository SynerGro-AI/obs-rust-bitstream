//! Safe pure LiveFeed (camera jump core).
//!
//! A feed owns a name and a source. The director jumps to it.
//! Empty names rejected. No unwraps.
//!
//! Differential: jump + current round-trips.

use crate::ai::AiAssistant;
use crate::input_device::InputDevice;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct LiveFeed {
    current: Option<String>,
}

impl LiveFeed {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jump(&mut self, name: &str) -> Result<(), FeedError> {
        if name.is_empty() {
            return Err(FeedError::EmptyName);
        }
        self.current = Some(name.to_string());
        Ok(())
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    /// Switch camera from an arrow key. Left and right cycle
    /// a small set. Returns the new current camera.
    pub fn from_key(&mut self, input: &InputDevice, cameras: &[&str]) -> Option<String> {
        if cameras.is_empty() {
            return None;
        }
        let key = input.last_key().unwrap_or("");
        let idx = cameras
            .iter()
            .position(|c| Some(*c) == self.current())
            .unwrap_or(0);
        let next = match key {
            "left" | "arrowleft" => {
                if idx == 0 { cameras.len() - 1 } else { idx - 1 }
            }
            "right" | "arrowright" => (idx + 1) % cameras.len(),
            _ => return self.current().map(|s| s.to_string()),
        };
        let name = cameras[next];
        let _ = self.jump(name);
        Some(name.to_string())
    }

    /// Switch camera from an AI line. If the line names a
    /// camera, jump to it.
    pub fn from_ai(&mut self, ai: &AiAssistant, line: &str, cameras: &[&str]) -> Option<String> {
        let r = ai.assist(line).to_lowercase();
        let l = line.to_lowercase();
        for cam in cameras {
            if r.contains(&cam.to_lowercase()) || l.contains(&cam.to_lowercase()) {
                let _ = self.jump(cam);
                return Some(cam.to_string());
            }
        }
        self.current().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let f = LiveFeed::new();
        assert!(f.current().is_none());
    }

    #[test]
    fn empty_name() {
        let mut f = LiveFeed::new();
        assert!(f.jump("").is_err());
    }

    #[test]
    fn differential() {
        let mut f = LiveFeed::new();
        f.jump("trump_cam").unwrap();
        assert_eq!(f.current(), Some("trump_cam"));
        f.clear();
        assert!(f.current().is_none());
    }

    #[test]
    fn from_key() {
        let mut f = LiveFeed::new();
        let cams = ["cam1", "cam2", "cam3"];
        f.jump("cam1").unwrap();
        let mut input = InputDevice::new();
        input.press("right");
        assert_eq!(f.from_key(&input, &cams), Some("cam2".to_string()));
    }

    #[test]
    fn from_ai() {
        let mut f = LiveFeed::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let cams = ["trump_cam", "desk"];
        assert_eq!(
            f.from_ai(&ai, "jump to trump_cam", &cams),
            Some("trump_cam".to_string())
        );
    }
}
