//! Safe pure MultiAngle (camera angles core).
//!
//! Maps a camera to its monitor angles. Empty names rejected.
//! No unwraps.
//!
//! Differential: add angles + list round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AngleError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct MultiAngle {
    angles: BTreeMap<String, Vec<String>>,
}

impl MultiAngle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, camera: &str, angle: &str) -> Result<(), AngleError> {
        if camera.is_empty() || angle.is_empty() {
            return Err(AngleError::EmptyName);
        }
        self.angles
            .entry(camera.to_string())
            .or_default()
            .push(angle.to_string());
        Ok(())
    }

    pub fn angles_of(&self, camera: &str) -> Option<&[String]> {
        self.angles.get(camera).map(|v| v.as_slice())
    }

    pub fn count(&self, camera: &str) -> usize {
        self.angles.get(camera).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = MultiAngle::new();
        assert_eq!(m.count("cam1"), 0);
    }

    #[test]
    fn empty_name() {
        let mut m = MultiAngle::new();
        assert!(m.add("", "wide").is_err());
    }

    #[test]
    fn differential() {
        let mut m = MultiAngle::new();
        m.add("cam1", "wide").unwrap();
        m.add("cam1", "close").unwrap();
        m.add("cam1", "overhead").unwrap();
        assert_eq!(m.count("cam1"), 3);
        assert_eq!(m.angles_of("cam1").unwrap()[1], "close");
    }
}
