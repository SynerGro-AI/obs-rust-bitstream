//! Safe pure AudioBackendFile (status → bytes and back).
//!
//! Writes an AudioBackendStatus and reads the live flag back.
//! Returns None if malformed. No unwraps.
//!
//! Differential: a live backend reads true.

use crate::audio_backend_status::AudioBackendStatus;

pub struct AudioBackendFile;

impl AudioBackendFile {
    /// Write the one-line report.
    pub fn write(status: &AudioBackendStatus) -> Vec<u8> {
        status.one_line().into_bytes()
    }

    /// Read the live flag. Returns None if malformed.
    pub fn live(buf: &[u8]) -> Option<bool> {
        let text = std::str::from_utf8(buf).ok()?;
        if text.contains("live=true") {
            return Some(true);
        }
        if text.contains("live=false") {
            return Some(false);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_false() {
        let s = AudioBackendStatus {
            backend: "alsa".to_string(),
            live: false,
        };
        assert_eq!(AudioBackendFile::live(&AudioBackendFile::write(&s)), Some(false));
    }

    #[test]
    fn write_read_true() {
        let s = AudioBackendStatus {
            backend: "cpal".to_string(),
            live: true,
        };
        assert_eq!(AudioBackendFile::live(&AudioBackendFile::write(&s)), Some(true));
    }

    #[test]
    fn bad_is_none() {
        assert!(AudioBackendFile::live(b"bad").is_none());
        assert!(AudioBackendFile::live(&[]).is_none());
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
        assert_eq!(AudioBackendFile::live(&AudioBackendFile::write(&a)), Some(false));
        assert_eq!(AudioBackendFile::live(&AudioBackendFile::write(&b)), Some(true));
    }
}
