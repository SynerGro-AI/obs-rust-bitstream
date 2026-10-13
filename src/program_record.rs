//! Safe pure ProgramRecord (program capture core).
//!
//! Records frames from the program output. start begins,
//! stop ends. No unwraps.
//!
//! Differential: start + record + stop round-trips.

#[derive(Clone, Debug, Default)]
pub struct ProgramRecord {
    recording: bool,
    frames: u64,
}

impl ProgramRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.recording = true;
        self.frames = 0;
    }

    pub fn stop(&mut self) {
        self.recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Record one frame. Returns false if not recording.
    pub fn record_frame(&mut self) -> bool {
        if self.recording {
            self.frames += 1;
            true
        } else {
            false
        }
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = ProgramRecord::new();
        assert!(!r.is_recording());
    }

    #[test]
    fn differential() {
        let mut r = ProgramRecord::new();
        r.start();
        assert!(r.record_frame());
        assert!(r.record_frame());
        assert_eq!(r.frames(), 2);
        r.stop();
        assert!(!r.record_frame());
    }
}
