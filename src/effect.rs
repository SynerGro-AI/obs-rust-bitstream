//! Safe pure Effect (scene effect core).
//!
//! An effect owns a name and an intensity. The director
//! triggers it from a scene. Empty names rejected. No unwraps.
//!
//! Differential: set intensity + active round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FxError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Effect {
    name: String,
    intensity: f32,
    active: bool,
}

impl Effect {
    pub fn new(name: &str) -> Result<Self, FxError> {
        if name.is_empty() {
            return Err(FxError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            intensity: 0.0,
            active: false,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn trigger(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
        self.active = self.intensity > 0.0;
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.intensity = 0.0;
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let e = Effect::new("western").unwrap();
        assert!(!e.active());
    }

    #[test]
    fn empty_name() {
        assert!(Effect::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut e = Effect::new("western").unwrap();
        e.trigger(0.8);
        assert!(e.active());
        assert!((e.intensity() - 0.8).abs() < 0.001);
        e.stop();
        assert!(!e.active());
    }
}
