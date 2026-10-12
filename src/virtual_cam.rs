//! Safe pure VirtualCam (OBS virtual camera core).
//!
//! A virtual camera owns a name and an active flag. Empty names
//! are rejected. No unwraps.
//!
//! Differential: start + is active round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualCamError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct VirtualCam {
    name: String,
    active: bool,
}

impl VirtualCam {
    pub fn new(name: &str) -> Result<Self, VirtualCamError> {
        if name.is_empty() {
            return Err(VirtualCamError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            active: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let v = VirtualCam::new("main").unwrap();
        assert_eq!(v.name(), "main");
        assert!(!v.active());
    }

    #[test]
    fn empty_name() {
        assert!(VirtualCam::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut v = VirtualCam::new("cam").unwrap();
        v.start();
        assert!(v.active());
        v.stop();
        assert!(!v.active());
    }
}
