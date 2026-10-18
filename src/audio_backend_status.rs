//! Safe pure AudioBackendStatus (backend + live).
//!
//! One line of the backend name and whether it is live.
//! No unwraps.
//!
//! Differential: a live backend reports live=true.

use crate::audio_platform::backend_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioBackendStatus {
    pub backend: String,
    pub live: bool,
}

impl AudioBackendStatus {
    /// Build from the current backend and a live flag.
    pub fn from(live: bool) -> Self {
        Self {
            backend: backend_name().to_string(),
            live,
        }
    }

    /// One line: "backend=N live=true/false"
    pub fn one_line(&self) -> String {
        format!("backend={} live={}", self.backend, self.live)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let s = AudioBackendStatus::from(false);
        assert!(!s.live);
        assert!(!s.backend.is_empty());
        assert!(s.one_line().contains("live=false"));
    }

    #[test]
    fn live_true() {
        let s = AudioBackendStatus::from(true);
        assert!(s.live);
        assert!(s.one_line().contains("live=true"));
    }

    #[test]
    fn differential() {
        let a = AudioBackendStatus::from(false);
        let b = AudioBackendStatus::from(true);
        assert!(!a.live);
        assert!(b.live);
        assert_ne!(a.one_line(), b.one_line());
    }
}
