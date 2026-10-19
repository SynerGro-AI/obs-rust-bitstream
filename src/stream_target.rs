//! Safe pure StreamTarget (multi-platform RTMP).
//!
//! Holds a platform, ingest URL, and stream key.
//! Covers YouTube, Twitch, Facebook, X, and custom.
//! Empty keys are rejected. No unwraps.
//!
//! Differential: a different platform produces a different endpoint.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    YouTube,
    Twitch,
    Facebook,
    X,
    Discord,
    Cloudflare,
    Custom,
}

impl Platform {
    pub fn ingest(&self) -> &'static str {
        match self {
            Platform::YouTube => "rtmp://a.rtmp.youtube.com/live2",
            Platform::Twitch => "rtmp://live.twitch.tv/app",
            Platform::Facebook => "rtmps://live-api-s.facebook.com:443/rtmp",
            Platform::X => "rtmp://ingest.pscp.tv:80/x",
            Platform::Discord => "discord://go-live",
            Platform::Cloudflare => "rtmps://live.cloudflare.com:443/live",
            Platform::Custom => "",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::YouTube => "youtube",
            Platform::Twitch => "twitch",
            Platform::Facebook => "facebook",
            Platform::X => "x",
            Platform::Discord => "discord",
            Platform::Cloudflare => "cloudflare",
            Platform::Custom => "custom",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamTarget {
    platform: Platform,
    url: String,
    key: String,
}

impl StreamTarget {
    /// Create a target for a known platform. None if the key is empty.
    pub fn new(platform: Platform, key: &str) -> Option<Self> {
        if key.is_empty() {
            return None;
        }
        Some(Self {
            url: platform.ingest().to_string(),
            platform,
            key: key.to_string(),
        })
    }

    /// Stream to a domain you own. Builds rtmp://domain/live.
    /// Returns None if the domain or key is empty.
    pub fn owned(domain: &str, key: &str) -> Option<Self> {
        if domain.is_empty() || key.is_empty() {
            return None;
        }
        let url = if domain.starts_with("rtmp://") || domain.starts_with("rtmps://") {
            domain.to_string()
        } else {
            format!("rtmp://{}/live", domain)
        };
        Some(Self {
            platform: Platform::Custom,
            url,
            key: key.to_string(),
        })
    }

    /// Create a custom target. None if either is empty.
    pub fn custom(url: &str, key: &str) -> Option<Self> {
        if url.is_empty() || key.is_empty() {
            return None;
        }
        Some(Self {
            platform: Platform::Custom,
            url: url.to_string(),
            key: key.to_string(),
        })
    }

    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub fn endpoint(&self) -> String {
        format!("{}/{}", self.url, self.key)
    }

    pub fn line(&self) -> String {
        format!("{} url={} key={}", self.platform.name(), self.url, self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platforms() {
        let yt = StreamTarget::new(Platform::YouTube, "yt-key").unwrap();
        let tw = StreamTarget::new(Platform::Twitch, "tw-key").unwrap();
        let fb = StreamTarget::new(Platform::Facebook, "fb-key").unwrap();
        let x = StreamTarget::new(Platform::X, "x-key").unwrap();
        let dc = StreamTarget::new(Platform::Discord, "dc-key").unwrap();
        let cf = StreamTarget::new(Platform::Cloudflare, "cf-key").unwrap();
        assert!(yt.endpoint().contains("youtube"));
        assert!(tw.endpoint().contains("twitch"));
        assert!(fb.endpoint().contains("facebook"));
        assert!(x.endpoint().contains("pscp"));
        assert!(dc.endpoint().contains("discord"));
        assert!(cf.endpoint().contains("cloudflare"));
    }

    #[test]
    fn owned_domain() {
        let t = StreamTarget::owned("stream.example.com", "my-key").unwrap();
        assert!(t.endpoint().contains("stream.example.com"));
        assert!(t.endpoint().contains("my-key"));
        assert!(t.endpoint().starts_with("rtmp://"));
    }

    #[test]
    fn empty_rejected() {
        assert!(StreamTarget::new(Platform::Twitch, "").is_none());
        assert!(StreamTarget::custom("", "key").is_none());
    }

    #[test]
    fn differential() {
        let a = StreamTarget::new(Platform::YouTube, "k").unwrap();
        let b = StreamTarget::new(Platform::Twitch, "k").unwrap();
        assert_ne!(a.endpoint(), b.endpoint());
    }
}
