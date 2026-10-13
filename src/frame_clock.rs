//! Safe pure FrameClock (frame timing core).
//!
//! Owns an fps and a frame count. tick advances one frame.
//! No unwraps.
//!
//! Differential: tick at 30 fps round-trips.

#[derive(Clone, Debug)]
pub struct FrameClock {
    fps: f32,
    frame: u64,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::new(30.0)
    }
}

impl FrameClock {
    pub fn new(fps: f32) -> Self {
        Self {
            fps: fps.max(1.0),
            frame: 0,
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Advance one frame. Returns the new frame number.
    pub fn tick(&mut self) -> u64 {
        self.frame += 1;
        self.frame
    }

    /// Seconds elapsed.
    pub fn seconds(&self) -> f32 {
        self.frame as f32 / self.fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = FrameClock::new(30.0);
        assert_eq!(c.fps(), 30.0);
        assert_eq!(c.frame(), 0);
    }

    #[test]
    fn differential() {
        let mut c = FrameClock::new(30.0);
        assert_eq!(c.tick(), 1);
        assert_eq!(c.tick(), 2);
        assert!((c.seconds() - 2.0 / 30.0).abs() < 0.001);
    }
}
