//! Safe pure BrowserSource (URL holder core).
//!
//! Holds a URL and a size. Empty URLs rejected.
//! No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserError {
    EmptyUrl,
    ZeroSize,
}

#[derive(Clone, Debug)]
pub struct BrowserSource {
    url: String,
    width: u32,
    height: u32,
}

impl BrowserSource {
    pub fn new(url: &str, width: u32, height: u32) -> Result<Self, BrowserError> {
        if url.is_empty() {
            return Err(BrowserError::EmptyUrl);
        }
        if width == 0 || height == 0 {
            return Err(BrowserError::ZeroSize);
        }
        Ok(Self {
            url: url.to_string(),
            width,
            height,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn set_url(&mut self, url: &str) -> Result<(), BrowserError> {
        if url.is_empty() {
            return Err(BrowserError::EmptyUrl);
        }
        self.url = url.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(BrowserSource::new("", 1920, 1080).is_err());
        assert!(BrowserSource::new("https://x.com", 0, 1080).is_err());
    }

    #[test]
    fn differential() {
        let mut b = BrowserSource::new("https://x.com", 1920, 1080).unwrap();
        assert_eq!(b.url(), "https://x.com");
        assert_eq!(b.size(), (1920, 1080));
        b.set_url("https://synapsis.school").unwrap();
        assert_eq!(b.url(), "https://synapsis.school");
    }
}
