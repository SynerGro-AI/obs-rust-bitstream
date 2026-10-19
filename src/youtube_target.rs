//! Safe pure YouTubeTarget (RTMP URL + stream key).
//!
//! Holds the YouTube ingest URL and stream key.
//! Builds the full endpoint. Empty keys are rejected.
//! No unwraps.
//!
//! Differential: a different key produces a different endpoint.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YouTubeTarget {
    url: String,
    key: String,
}

impl YouTubeTarget {
    pub const DEFAULT_URL: &'static str = "rtmp://a.rtmp.youtube.com/live2";

    /// Create a target. Returns None if the key is empty.
    pub fn new(key: &str) -> Option<Self> {
        if key.is_empty() {
            return None;
        }
        Some(Self {
            url: Self::DEFAULT_URL.to_string(),
            key: key.to_string(),
        })
    }

    /// Create with a custom URL. Returns None if either is empty.
    pub fn with_url(url: &str, key: &str) -> Option<Self> {
        if url.is_empty() || key.is_empty() {
            return None;
        }
        Some(Self {
            url: url.to_string(),
            key: key.to_string(),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// Full endpoint: url/key
    pub fn endpoint(&self) -> String {
        format!("{}/{}", self.url, self.key)
    }

    /// One line: "youtube url=... key=..."
    pub fn line(&self) -> String {
        format!("youtube url={} key={}", self.url, self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let t = YouTubeTarget::new("abcd-1234").unwrap();
        assert!(t.endpoint().contains("youtube.com"));
        assert!(t.endpoint().contains("abcd-1234"));
        assert!(t.line().contains("youtube"));
    }

    #[test]
    fn empty_rejected() {
        assert!(YouTubeTarget::new("").is_none());
        assert!(YouTubeTarget::with_url("", "key").is_none());
    }

    #[test]
    fn differential() {
        let a = YouTubeTarget::new("key-a").unwrap();
        let b = YouTubeTarget::new("key-b").unwrap();
        assert_ne!(a.endpoint(), b.endpoint());
    }
}
