//! Safe pure Performance (singer + music video core).
//!
//! A performance owns a singer camera and a music video
//! source. When live, the camera frames the singer and the
//! video renders behind. No unwraps.
//!
//! Differential: go live + frame round-trips.

use crate::live_camera::LiveCamera;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PerformanceError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Performance {
    name: String,
    video: String,
    live: bool,
}

impl Performance {
    pub fn new(name: &str) -> Result<Self, PerformanceError> {
        if name.is_empty() {
            return Err(PerformanceError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            video: String::new(),
            live: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_video(&mut self, src: &str) {
        self.video = src.to_string();
    }

    pub fn video(&self) -> &str {
        &self.video
    }

    pub fn go_live(&mut self) {
        self.live = true;
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    /// Composite: video background color plus the singer frame.
    /// Returns (background, singer).
    pub fn frame(&self, camera: &LiveCamera) -> ((f32, f32, f32), (f32, f32, f32)) {
        let bg = if self.video.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            (0.4, 0.1, 0.5)
        };
        let singer = if self.live { camera.frame() } else { (0.0, 0.0, 0.0) };
        (bg, singer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let p = Performance::new("set").unwrap();
        assert!(!p.is_live());
    }

    #[test]
    fn empty_name() {
        assert!(Performance::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut p = Performance::new("set").unwrap();
        p.set_video("music_video.mp4");
        p.go_live();
        let mut cam = LiveCamera::new("singer").unwrap();
        cam.go_live();
        let (bg, singer) = p.frame(&cam);
        assert_eq!(bg, (0.4, 0.1, 0.5));
        assert_eq!(singer, (0.2, 0.4, 0.8));
    }
}
