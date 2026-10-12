//! Safe pure Environment (world physics governor).
//!
//! An environment owns a name, air density, wind, and a gravity
//! override. It writes those values into a PhysicsSettings so
//! any simulation is governed by the world. Empty names rejected.
//! No unwraps.
//!
//! Differential: set density + write into settings round-trips.

use crate::physics::PhysicsSettings;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Environment {
    name: String,
    air_density: f32,
    wind_x: f32,
    gravity: f32,
}

impl Environment {
    pub fn new(name: &str) -> Result<Self, EnvironmentError> {
        if name.is_empty() {
            return Err(EnvironmentError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            air_density: 1.225,
            wind_x: 0.0,
            gravity: -9.81,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn air_density(&self) -> f32 {
        self.air_density
    }

    pub fn set_air_density(&mut self, d: f32) {
        self.air_density = d.max(0.0);
    }

    pub fn wind_x(&self) -> f32 {
        self.wind_x
    }

    pub fn set_wind_x(&mut self, w: f32) {
        self.wind_x = w;
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity(&mut self, g: f32) {
        self.gravity = g;
    }

    /// Write the environment's laws into a physics settings bag.
    pub fn govern(&self, settings: &mut PhysicsSettings) {
        settings.set_gravity(self.gravity);
        settings.set_friction((self.air_density * 0.1).min(1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let e = Environment::new("earth").unwrap();
        assert_eq!(e.name(), "earth");
        assert!((e.air_density() - 1.225).abs() < 0.001);
    }

    #[test]
    fn empty_name() {
        assert!(Environment::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut e = Environment::new("mars").unwrap();
        e.set_gravity(-3.71);
        e.set_air_density(0.02);
        let mut s = PhysicsSettings::new();
        e.govern(&mut s);
        assert!((s.gravity() + 3.71).abs() < 0.001);
        assert!(s.friction() < 0.01);
    }
}
