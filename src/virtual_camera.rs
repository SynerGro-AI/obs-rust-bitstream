//! Safe pure VirtualCamera (virtual cam core).
//!
//! Sends the program output to a virtual camera device.
//! start begins, stop ends. No unwraps.
//!
//! Differential: start + frames + stop round-trips.

#[derive(Clone, Debug, Default)]
pub struct VirtualCamera {
    running: bool,
    frames: u64,
}

impl VirtualCamera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.running = true;
        self.frames = 0;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Send one frame to the virtual camera. Returns false if off.
    pub fn send_frame(&mut self) -> bool {
        if self.running {
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
        let v = VirtualCamera::new();
        assert!(!v.is_running());
    }

    #[test]
    fn differential() {
        let mut v = VirtualCamera::new();
        v.start();
        assert!(v.send_frame());
        assert!(v.send_frame());
        assert_eq!(v.frames(), 2);
        v.stop();
        assert!(!v.send_frame());
    }
}
