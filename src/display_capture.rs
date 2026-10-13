//! Safe pure DisplayCapture (monitor holder core).
//!
//! Holds a monitor index and a cursor flag. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug)]
pub struct DisplayCapture {
    monitor: u32,
    cursor: bool,
}

impl DisplayCapture {
    pub fn new(monitor: u32, cursor: bool) -> Self {
        Self { monitor, cursor }
    }

    pub fn monitor(&self) -> u32 {
        self.monitor
    }

    pub fn cursor(&self) -> bool {
        self.cursor
    }

    pub fn set_monitor(&mut self, monitor: u32) {
        self.monitor = monitor;
    }

    pub fn set_cursor(&mut self, cursor: bool) {
        self.cursor = cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let d = DisplayCapture::new(0, true);
        assert_eq!(d.monitor(), 0);
        assert!(d.cursor());
    }

    #[test]
    fn differential() {
        let mut d = DisplayCapture::new(0, false);
        d.set_monitor(1);
        d.set_cursor(true);
        assert_eq!(d.monitor(), 1);
        assert!(d.cursor());
    }
}
