//! Safe pure RtmpStart (service → RTMP stream).
//!
//! Builds an RtmpStream URL from the service name and
//! connects it. Empty name is rejected. No unwraps.
//!
//! Differential: a different name produces a different URL.

use crate::rtmp_stream::{RtmpError, RtmpStream};

pub struct RtmpStart;

impl RtmpStart {
    /// Build the URL and connect. Returns the connected stream.
    pub fn start(name: &str, key: &str) -> Result<RtmpStream, RtmpError> {
        if name.is_empty() {
            return Err(RtmpError::EmptyUrl);
        }
        let url = if key.is_empty() {
            format!("rtmp://{}/live", name)
        } else {
            format!("rtmp://{}/live/{}", name, key)
        };
        let mut stream = RtmpStream::new(&url)?;
        stream.connect();
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts() {
        let s = RtmpStart::start("live.twitch.tv", "abc").unwrap();
        assert!(s.is_active());
        assert_eq!(s.url(), "rtmp://live.twitch.tv/live/abc");
    }

    #[test]
    fn empty_rejected() {
        assert!(RtmpStart::start("", "abc").is_err());
    }

    #[test]
    fn no_key() {
        let s = RtmpStart::start("live.youtube.com", "").unwrap();
        assert_eq!(s.url(), "rtmp://live.youtube.com/live");
    }

    #[test]
    fn differential() {
        let a = RtmpStart::start("twitch", "a").unwrap();
        let b = RtmpStart::start("youtube", "b").unwrap();
        assert_ne!(a.url(), b.url());
    }
}
