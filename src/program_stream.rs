//! Safe pure ProgramStream (program stream core).
//!
//! Streams the program output to a target. start begins,
//! stop ends. Empty targets rejected. No unwraps.
//!
//! Differential: start + frames + stop round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamError {
    EmptyTarget,
}

#[derive(Clone, Debug, Default)]
pub struct ProgramStream {
    target: String,
    streaming: bool,
    frames: u64,
}

impl ProgramStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, target: &str) -> Result<(), StreamError> {
        if target.is_empty() {
            return Err(StreamError::EmptyTarget);
        }
        self.target = target.to_string();
        self.streaming = true;
        self.frames = 0;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.streaming = false;
    }

    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    /// Send one frame. Returns false if not streaming.
    pub fn send_frame(&mut self) -> bool {
        if self.streaming {
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
        let s = ProgramStream::new();
        assert!(!s.is_streaming());
    }

    #[test]
    fn empty_target() {
        let mut s = ProgramStream::new();
        assert!(s.start("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = ProgramStream::new();
        s.start("rtmp://live").unwrap();
        assert!(s.send_frame());
        assert_eq!(s.frames(), 1);
        assert_eq!(s.target(), "rtmp://live");
        s.stop();
        assert!(!s.send_frame());
    }
}
