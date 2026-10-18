//! Safe pure AudioBackendRound (write → read).
//!
//! Writes an AudioBackendStatus and reads the live flag back.
//! Returns true if the value matches. No unwraps.
//!
//! Differential: a live backend reads true; a fallback reads false.

use crate::audio_backend_file::AudioBackendFile;
use crate::audio_backend_status::AudioBackendStatus;

pub struct AudioBackendRound;

impl AudioBackendRound {
    /// True if the written flag reads back the same.
    pub fn matches(status: &AudioBackendStatus) -> bool {
        let buf = AudioBackendFile::write(status);
        AudioBackendFile::live(&buf) == Some(status.live)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_false() {
        let s = AudioBackendStatus {
            backend: "alsa".to_string(),
            live: false,
        };
        assert!(AudioBackendRound::matches(&s));
    }

    #[test]
    fn round_trips_true() {
        let s = AudioBackendStatus {
            backend: "cpal".to_string(),
            live: true,
        };
        assert!(AudioBackendRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = AudioBackendStatus {
            backend: "alsa".to_string(),
            live: false,
        };
        let b = AudioBackendStatus {
            backend: "cpal".to_string(),
            live: true,
        };
        assert!(AudioBackendRound::matches(&a));
        assert!(AudioBackendRound::matches(&b));
        assert_ne!(a.live, b.live);
    }
}
