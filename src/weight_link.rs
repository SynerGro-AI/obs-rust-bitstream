//! Safe pure WeightLink (live weights into scene objects).
//!
//! Applies named weights to light intensity, physics
//! gravity, and bone translation. Missing weights are
//! skipped. No unwraps.
//!
//! Differential: a weight of 2.0 sets light intensity.

use crate::bone::Bone;
use crate::light::Light;
use crate::live_weights::LiveWeights;
use crate::physics::PhysicsSettings;

pub struct WeightLink;

impl WeightLink {
    /// Set light intensity from "light.<name>.intensity".
    pub fn light(light: &mut Light, weights: &LiveWeights) {
        let key = format!("light.{}.intensity", light.name());
        if let Ok(v) = weights.get(&key) {
            light.set_intensity(v);
        }
    }

    /// Set physics gravity from "physics.gravity".
    pub fn physics(phys: &mut PhysicsSettings, weights: &LiveWeights) {
        if let Ok(v) = weights.get("physics.gravity") {
            phys.set_gravity(v);
        }
        if let Ok(v) = weights.get("physics.friction") {
            phys.set_friction(v);
        }
        if let Ok(v) = weights.get("physics.mass") {
            phys.set_mass(v);
        }
    }

    /// Set bone translation from "bone.<name>.x/y/z".
    pub fn bone(bone: &mut Bone, weights: &LiveWeights) {
        let nx = format!("bone.{}.x", bone.name());
        let ny = format!("bone.{}.y", bone.name());
        let nz = format!("bone.{}.z", bone.name());
        let x = weights.get(&nx).unwrap_or(0.0);
        let y = weights.get(&ny).unwrap_or(0.0);
        let z = weights.get(&nz).unwrap_or(0.0);
        if weights.get(&nx).is_ok() || weights.get(&ny).is_ok() || weights.get(&nz).is_ok() {
            bone.set_local_translation(x, y, z);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn differential() {
        let mut w = LiveWeights::new();
        w.set("light.main.intensity", 2.0).unwrap();
        let mut light = Light::new("main").unwrap();
        WeightLink::light(&mut light, &w);
        assert_eq!(light.intensity(), 2.0);

        w.set("physics.gravity", 9.8).unwrap();
        let mut phys = PhysicsSettings::new();
        WeightLink::physics(&mut phys, &w);
        assert_eq!(phys.gravity(), 9.8);

        w.set("bone.arm.x", 1.0).unwrap();
        let mut bone = Bone::new("arm").unwrap();
        WeightLink::bone(&mut bone, &w);
        // translation was set; no panic
        let _ = bone.local();
    }
}
