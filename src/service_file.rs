//! Safe pure ServiceFile (service name → bytes and back).
//!
//! Writes a ServiceStatus name and reads it back.
//! Returns None if the buffer is malformed. No unwraps.
//!
//! Differential: a different name reads differently.

use crate::service_status::ServiceStatus;

pub struct ServiceFile;

impl ServiceFile {
    /// Write the name as "svc=NAME".
    pub fn write(status: &ServiceStatus) -> Vec<u8> {
        format!("svc={}", status.name).into_bytes()
    }

    /// Read the name. Returns None if malformed.
    pub fn read(buf: &[u8]) -> Option<String> {
        let text = std::str::from_utf8(buf).ok()?;
        if !text.starts_with("svc=") {
            return None;
        }
        Some(text[4..].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let s = ServiceStatus {
            name: "twitch".to_string(),
            id: "rtmp_common".to_string(),
        };
        let buf = ServiceFile::write(&s);
        assert_eq!(ServiceFile::read(&buf), Some("twitch".to_string()));
    }

    #[test]
    fn empty_reads_empty() {
        let s = ServiceStatus {
            name: String::new(),
            id: String::new(),
        };
        let buf = ServiceFile::write(&s);
        assert_eq!(ServiceFile::read(&buf), Some(String::new()));
    }

    #[test]
    fn bad_is_none() {
        assert!(ServiceFile::read(b"bad").is_none());
        assert!(ServiceFile::read(&[]).is_none());
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
        assert_ne!(
            ServiceFile::read(&ServiceFile::write(&a)),
            ServiceFile::read(&ServiceFile::write(&b))
        );
    }
}
