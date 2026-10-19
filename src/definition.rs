//! Safe pure Definition (pixel count and quality).
//!
//! Holds a pixel count (width x height) and a quality level.
//! Quality maps to a sharpness and a bitrate suggestion.
//! Empty or zero sizes are rejected. No unwraps.
//!
//! Differential: a higher quality produces a higher bitrate.

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    width: u32,
    height: u32,
    quality: f32,
}

impl Definition {
    /// Create a definition. None if width or height is zero.
    pub fn new(width: u32, height: u32, quality: f32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            quality: quality.clamp(0.0, 100.0),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Total pixel count.
    pub fn pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn quality(&self) -> f32 {
        self.quality
    }

    /// Set the pixel count. None if either is zero.
    pub fn set_pixels(&mut self, width: u32, height: u32) -> Option<()> {
        if width == 0 || height == 0 {
            return None;
        }
        self.width = width;
        self.height = height;
        Some(())
    }

    /// Set the quality. Clamped to 0..=100.
    pub fn set_quality(&mut self, quality: f32) {
        self.quality = quality.clamp(0.0, 100.0);
    }

    /// Suggested bitrate in kbps. Scales with pixels and quality.
    pub fn bitrate_kbps(&self) -> u32 {
        let base = (self.pixels() as f32 / 1000.0) * (self.quality / 100.0);
        base.max(100.0) as u32
    }

    /// Sharpness 0.0..=1.0 from quality.
    pub fn sharpness(&self) -> f32 {
        self.quality / 100.0
    }

    /// One line: "WxH q=N pixels=N bitrate=N"
    pub fn line(&self) -> String {
        format!("{}x{} q={:.0} pixels={} bitrate={}", self.width, self.height, self.quality, self.pixels(), self.bitrate_kbps())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates() {
        let d = Definition::new(1920, 1080, 80.0).unwrap();
        assert_eq!(d.pixels(), 1920 * 1080);
        assert!(d.bitrate_kbps() > 100);
        assert!(d.line().contains("1920x1080"));
    }

    #[test]
    fn zero_rejected() {
        assert!(Definition::new(0, 1080, 50.0).is_none());
    }

    #[test]
    fn set_pixels() {
        let mut d = Definition::new(1280, 720, 50.0).unwrap();
        d.set_pixels(1920, 1080).unwrap();
        assert_eq!(d.pixels(), 1920 * 1080);
    }

    #[test]
    fn differential() {
        let a = Definition::new(1920, 1080, 50.0).unwrap();
        let b = Definition::new(1920, 1080, 90.0).unwrap();
        assert!(b.bitrate_kbps() > a.bitrate_kbps());
    }
}
