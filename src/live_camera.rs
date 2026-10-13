//! Safe pure LiveCamera (news camera core).
//!
//! A camera owns a name and a live flag. When live it
//! produces a frame color. Empty names rejected. No unwraps.
//!
//! Differential: go live + frame round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct LiveCamera {
    name: String,
    live: bool,
    pan: f32,
    tilt: f32,
    zoom: f32,
}

impl LiveCamera {
    pub fn new(name: &str) -> Result<Self, CameraError> {
        if name.is_empty() {
            return Err(CameraError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            live: false,
            pan: 0.0,
            tilt: 0.0,
            zoom: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn go_live(&mut self) {
        self.live = true;
    }

    pub fn stop(&mut self) {
        self.live = false;
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    pub fn set_pan(&mut self, p: f32) {
        self.pan = p.clamp(-1.0, 1.0);
    }

    pub fn pan(&self) -> f32 {
        self.pan
    }

    pub fn set_tilt(&mut self, t: f32) {
        self.tilt = t.clamp(-1.0, 1.0);
    }

    pub fn tilt(&self) -> f32 {
        self.tilt
    }

    pub fn set_zoom(&mut self, z: f32) {
        self.zoom = z.clamp(1.0, 10.0);
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Frame color when live. Black when off.
    pub fn frame(&self) -> (f32, f32, f32) {
        if self.live {
            (0.2, 0.4, 0.8)
        } else {
            (0.0, 0.0, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = LiveCamera::new("desk").unwrap();
        assert!(!c.is_live());
    }

    #[test]
    fn empty_name() {
        assert!(LiveCamera::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut c = LiveCamera::new("desk").unwrap();
        c.go_live();
        c.set_zoom(2.0);
        assert!(c.is_live());
        assert_eq!(c.frame(), (0.2, 0.4, 0.8));
        c.stop();
        assert_eq!(c.frame(), (0.0, 0.0, 0.0));
    }
}
