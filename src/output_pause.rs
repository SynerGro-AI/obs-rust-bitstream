//! Safe pure OutputPause (OBS output pause).
//!
//! Parsed from libobs/obs-output.c.
//! Tracks pause start and end timestamps.
//! Offset accumulates the paused duration.
//! Only outputs with CAN_PAUSE can pause. No unwraps.
//!
//! Differential: a paused interval increases the offset.

use crate::output_info::OutputInfo;

/// Flag from OBS. Not previously defined in output_info.
pub const OUTPUT_CAN_PAUSE: u32 = 1 << 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PauseError {
    CannotPause,
    AlreadyPaused,
    NotPaused,
}

#[derive(Clone, Debug, Default)]
pub struct OutputPause {
    ts_start: u64,
    ts_end: u64,
    ts_offset: u64,
    last_video_ts: u64,
    paused: bool,
}

impl OutputPause {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn offset(&self) -> u64 {
        self.ts_offset
    }

    pub fn last_video_ts(&self) -> u64 {
        self.last_video_ts
    }

    pub fn set_last_video_ts(&mut self, ts: u64) {
        self.last_video_ts = ts;
    }

    /// Begin a pause. Only outputs with CAN_PAUSE can pause.
    /// Returns error if already paused.
    pub fn start(&mut self, output: &OutputInfo, now: u64) -> Result<(), PauseError> {
        if output.flags() & OUTPUT_CAN_PAUSE == 0 {
            return Err(PauseError::CannotPause);
        }
        if self.paused {
            return Err(PauseError::AlreadyPaused);
        }
        self.ts_start = now;
        self.ts_end = 0;
        self.paused = true;
        Ok(())
    }

    /// End a pause. Accumulates the duration into the offset.
    /// Returns error if not paused.
    pub fn end(&mut self, now: u64) -> Result<u64, PauseError> {
        if !self.paused {
            return Err(PauseError::NotPaused);
        }
        self.ts_end = now;
        let duration = now.saturating_sub(self.ts_start);
        self.ts_offset = self.ts_offset.saturating_add(duration);
        self.paused = false;
        Ok(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};

    fn pausable() -> OutputInfo {
        OutputInfo::new(
            "rec",
            "Recording",
            OUTPUT_AV | OUTPUT_ENCODED | OUTPUT_CAN_PAUSE,
        )
        .unwrap()
    }

    #[test]
    fn start_end() {
        let out = pausable();
        let mut p = OutputPause::new();
        p.start(&out, 1000).unwrap();
        assert!(p.is_paused());
        let d = p.end(1500).unwrap();
        assert_eq!(d, 500);
        assert_eq!(p.offset(), 500);
        assert!(!p.is_paused());
    }

    #[test]
    fn rejects_without_flag() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut p = OutputPause::new();
        assert!(p.start(&out, 0).is_err());
    }

    #[test]
    fn double_start() {
        let out = pausable();
        let mut p = OutputPause::new();
        p.start(&out, 0).unwrap();
        assert!(p.start(&out, 1).is_err());
    }

    #[test]
    fn differential() {
        let out = pausable();
        let mut a = OutputPause::new();
        let mut b = OutputPause::new();
        a.start(&out, 0).unwrap();
        a.end(100).unwrap();
        b.start(&out, 0).unwrap();
        b.end(300).unwrap();
        assert!(b.offset() > a.offset());
    }
}
