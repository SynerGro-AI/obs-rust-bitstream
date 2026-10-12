//! Safe pure CameraCut (multi-camera switch core).
//!
//! A cut switches the active camera in a CameraRig at a time.
//! Empty names are rejected. No unwraps.
//!
//! Differential: apply a cut + active name round-trips.

use crate::camera_rig::{CameraRig, CameraRigError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CutError {
    EmptyName,
    Missing,
}

impl From<CameraRigError> for CutError {
    fn from(e: CameraRigError) -> Self {
        match e {
            CameraRigError::EmptyName => CutError::EmptyName,
            CameraRigError::Missing => CutError::Missing,
            CameraRigError::Duplicate => CutError::Missing,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CameraCut {
    time_secs: f32,
    camera: String,
}

impl CameraCut {
    pub fn new(time_secs: f32, camera: &str) -> Result<Self, CutError> {
        if camera.is_empty() {
            return Err(CutError::EmptyName);
        }
        Ok(Self {
            time_secs,
            camera: camera.to_string(),
        })
    }

    pub fn time_secs(&self) -> f32 {
        self.time_secs
    }

    pub fn camera(&self) -> &str {
        &self.camera
    }

    /// Apply the cut if the current time has reached it.
    pub fn apply(&self, rig: &mut CameraRig, time: f32) -> Result<bool, CutError> {
        if time >= self.time_secs {
            rig.set_active(&self.camera)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_switches() {
        let mut rig = CameraRig::new();
        rig.add("wide").unwrap();
        rig.add("hero").unwrap();
        let cut = CameraCut::new(1.0, "hero").unwrap();
        assert!(cut.apply(&mut rig, 1.0).unwrap());
        assert_eq!(rig.active_name(), Some("hero"));
    }

    #[test]
    fn empty_name() {
        assert!(CameraCut::new(0.0, "").is_err());
    }

    #[test]
    fn differential() {
        let mut rig = CameraRig::new();
        rig.add("a").unwrap();
        rig.add("b").unwrap();
        let cut = CameraCut::new(0.5, "b").unwrap();
        assert!(!cut.apply(&mut rig, 0.4).unwrap());
        assert!(cut.apply(&mut rig, 0.5).unwrap());
        assert_eq!(rig.active_name(), Some("b"));
    }
}
