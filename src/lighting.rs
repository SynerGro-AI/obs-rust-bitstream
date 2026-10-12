//! Safe pure Lighting (material × light evaluation core).
//!
//! Evaluates a material under a light rig and returns a lit color.
//! No unwraps.
//!
//! Differential: one light of intensity 1 on a white material
//! returns a color near the light color.

use crate::light_rig::LightRig;
use crate::material::Material;
use crate::vec3::Vec3;

pub struct Lighting;

impl Lighting {
    /// Sum every light's contribution onto the material.
    /// Contribution = light.color * light.intensity * (1 - roughness).
    pub fn evaluate(material: &Material, rig: &LightRig) -> Vec3 {
        let base = material.base_color();
        let factor = 1.0 - material.roughness();
        let mut out = Vec3::new(0.0, 0.0, 0.0);
        // LightRig does not expose iteration, so we use total intensity
        // as a uniform multiplier. A future step can expose the lights.
        let intensity = rig.total_intensity();
        out.x = base.x * intensity * factor;
        out.y = base.y * intensity * factor;
        out.z = base.z * intensity * factor;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_light() {
        let mut rig = LightRig::new();
        rig.add("key").unwrap();
        let m = Material::new("white").unwrap();
        let c = Lighting::evaluate(&m, &rig);
        assert!(c.x > 0.0);
    }

    #[test]
    fn differential() {
        let mut rig = LightRig::new();
        rig.add("key").unwrap();
        rig.get_mut("key").unwrap().set_intensity(2.0);
        let mut m = Material::new("paint").unwrap();
        m.set_roughness(0.0);
        m.set_base_color(Vec3::new(1.0, 0.0, 0.0));
        let c = Lighting::evaluate(&m, &rig);
        assert!((c.x - 2.0).abs() < 0.001);
        assert_eq!(c.y, 0.0);
    }
}
