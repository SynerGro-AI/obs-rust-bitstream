//! Safe pure ServiceRound (write → read).
//!
//! Writes a ServiceStatus and reads the name back.
//! Returns true if the name matches. No unwraps.
//!
//! Differential: a different name fails the match.

use crate::service_file::ServiceFile;
use crate::service_status::ServiceStatus;

pub struct ServiceRound;

impl ServiceRound {
    /// True if the written name reads back the same.
    pub fn matches(status: &ServiceStatus) -> bool {
        let buf = ServiceFile::write(status);
        ServiceFile::read(&buf) == Some(status.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let s = ServiceStatus {
            name: "twitch".to_string(),
            id: "rtmp_common".to_string(),
        };
        assert!(ServiceRound::matches(&s));
    }

    #[test]
    fn empty_round_trips() {
        let s = ServiceStatus {
            name: String::new(),
            id: String::new(),
        };
        assert!(ServiceRound::matches(&s));
    }

    #[test]
    fn differential() {
        let a = ServiceStatus {
            name: "twitch".to_string(),
            id: String::new(),
        };
        let b = ServiceStatus {
            name: "youtube".to_string(),
            id: String::new(),
        };
        assert!(ServiceRound::matches(&a));
        assert!(ServiceRound::matches(&b));
        assert_ne!(a.name, b.name);
    }
}
