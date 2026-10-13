//! Safe pure Studio (program / preview core).
//!
//! Owns a program monitor and a preview monitor. swap
//! exchanges them. No unwraps.
//!
//! Differential: set + swap round-trips.

use crate::multi_monitor::MultiMonitor;
use crate::studio_transition::StudioTransition;

#[derive(Clone, Debug, Default)]
pub struct Studio {
    screens: MultiMonitor,
    transition: StudioTransition,
}

impl Studio {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_program(&mut self, scene: &str) {
        let _ = self.screens.assign("program", scene);
    }

    pub fn set_preview(&mut self, scene: &str) {
        let _ = self.screens.assign("preview", scene);
    }

    pub fn program(&self) -> Option<&str> {
        self.screens.showing("program")
    }

    pub fn preview(&self) -> Option<&str> {
        self.screens.showing("preview")
    }

    /// Swap program and preview.
    pub fn swap(&mut self) {
        let prog = self.program().unwrap_or("").to_string();
        let prev = self.preview().unwrap_or("").to_string();
        if !prev.is_empty() {
            self.set_program(&prev);
        }
        if !prog.is_empty() {
            self.set_preview(&prog);
        }
    }

    /// Start a fade from preview to program.
    pub fn fade(&mut self) {
        self.transition.start();
    }

    /// Advance the fade. Returns true when complete.
    pub fn fade_step(&mut self, dt: f32) -> bool {
        self.transition.step(dt)
    }

    pub fn fading(&self) -> bool {
        !self.transition.done() && self.transition.progress() > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = Studio::new();
        assert!(s.program().is_none());
    }

    #[test]
    fn differential() {
        let mut s = Studio::new();
        s.set_program("wide");
        s.set_preview("close");
        assert_eq!(s.program(), Some("wide"));
        s.swap();
        assert_eq!(s.program(), Some("close"));
        assert_eq!(s.preview(), Some("wide"));
    }

    #[test]
    fn fade() {
        let mut s = Studio::new();
        s.fade();
        assert!(!s.fade_step(0.3));
        assert!(s.fade_step(0.8));
    }
}
