//! Safe pure Multiview (all cameras core).
//!
//! Holds the cameras to show and which one is live.
//! Empty names rejected. No unwraps.
//!
//! Differential: add + set live round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct Multiview {
    cameras: Vec<String>,
    live: Option<String>,
}

impl Multiview {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, camera: &str) -> Result<(), ViewError> {
        if camera.is_empty() {
            return Err(ViewError::EmptyName);
        }
        self.cameras.push(camera.to_string());
        Ok(())
    }

    pub fn set_live(&mut self, camera: &str) -> Result<(), ViewError> {
        if !self.cameras.iter().any(|c| c == camera) {
            return Err(ViewError::Missing);
        }
        self.live = Some(camera.to_string());
        Ok(())
    }

    pub fn live(&self) -> Option<&str> {
        self.live.as_deref()
    }

    pub fn cameras(&self) -> &[String] {
        &self.cameras
    }

    pub fn count(&self) -> usize {
        self.cameras.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let v = Multiview::new();
        assert_eq!(v.count(), 0);
    }

    #[test]
    fn empty_name() {
        let mut v = Multiview::new();
        assert!(v.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut v = Multiview::new();
        v.add("cam1").unwrap();
        v.add("cam2").unwrap();
        v.add("cam3").unwrap();
        v.set_live("cam2").unwrap();
        assert_eq!(v.count(), 3);
        assert_eq!(v.live(), Some("cam2"));
    }
}
