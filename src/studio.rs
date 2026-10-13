//! Safe pure Studio (program / preview core).
//!
//! Owns a program monitor and a preview monitor. swap
//! exchanges them. No unwraps.
//!
//! Differential: set + swap round-trips.

use crate::multi_monitor::MultiMonitor;

#[derive(Clone, Debug, Default)]
pub struct Studio {
    screens: MultiMonitor,
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
}
