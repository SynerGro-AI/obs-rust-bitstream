//! Safe pure OutputInfo (OBS obs_output core).
//!
//! Parsed from libobs/obs-output.h.
//! Flags match OBS: VIDEO, AUDIO, AV, ENCODED, SERVICE.
//! An output holds a name, flags, and a running state.
//! start/stop round-trip. No unwraps.
//!
//! Differential: start changes the running flag.

/// Flags from OBS obs_output_info.flags.
pub const OUTPUT_VIDEO: u32 = 1 << 0;
pub const OUTPUT_AUDIO: u32 = 1 << 1;
pub const OUTPUT_AV: u32 = OUTPUT_VIDEO | OUTPUT_AUDIO;
pub const OUTPUT_ENCODED: u32 = 1 << 2;
pub const OUTPUT_SERVICE: u32 = 1 << 3;
pub const OUTPUT_MULTI_TRACK: u32 = 1 << 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputError {
    EmptyName,
    EmptyId,
}

#[derive(Clone, Debug)]
pub struct OutputInfo {
    id: String,
    name: String,
    flags: u32,
    running: bool,
    bytes: u64,
}

impl OutputInfo {
    /// Create an output. Empty id or name is rejected.
    pub fn new(id: &str, name: &str, flags: u32) -> Result<Self, OutputError> {
        if id.is_empty() {
            return Err(OutputError::EmptyId);
        }
        if name.is_empty() {
            return Err(OutputError::EmptyName);
        }
        Ok(Self {
            id: id.to_string(),
            name: name.to_string(),
            flags,
            running: false,
            bytes: 0,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn has_video(&self) -> bool {
        self.flags & OUTPUT_VIDEO != 0
    }

    pub fn has_audio(&self) -> bool {
        self.flags & OUTPUT_AUDIO != 0
    }

    pub fn is_encoded(&self) -> bool {
        self.flags & OUTPUT_ENCODED != 0
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Start the output. Returns true if it was stopped.
    pub fn start(&mut self) -> bool {
        if self.running {
            return false;
        }
        self.running = true;
        self.bytes = 0;
        true
    }

    /// Stop the output. Returns true if it was running.
    pub fn stop(&mut self) -> bool {
        if !self.running {
            return false;
        }
        self.running = false;
        true
    }

    /// Count bytes while running. Returns false if stopped.
    pub fn add_bytes(&mut self, n: u64) -> bool {
        if !self.running {
            return false;
        }
        self.bytes = self.bytes.saturating_add(n);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_match_obs() {
        assert_eq!(OUTPUT_VIDEO, 1);
        assert_eq!(OUTPUT_AUDIO, 2);
        assert_eq!(OUTPUT_AV, 3);
        assert_eq!(OUTPUT_ENCODED, 4);
    }

    #[test]
    fn empty_rejected() {
        assert!(OutputInfo::new("", "rec", OUTPUT_AV).is_err());
        assert!(OutputInfo::new("rec", "", OUTPUT_AV).is_err());
    }

    #[test]
    fn start_stop() {
        let mut o = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        assert!(o.start());
        assert!(o.is_running());
        assert!(o.add_bytes(100));
        assert_eq!(o.bytes(), 100);
        assert!(o.stop());
        assert!(!o.add_bytes(50));
        assert_eq!(o.bytes(), 100);
    }

    #[test]
    fn differential() {
        let mut a = OutputInfo::new("a", "A", OUTPUT_VIDEO).unwrap();
        let b = OutputInfo::new("b", "B", OUTPUT_AUDIO).unwrap();
        a.start();
        assert!(a.is_running());
        assert!(!b.is_running());
        assert!(a.has_video());
        assert!(!b.has_video());
    }
}
