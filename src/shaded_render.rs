//! Safe pure ShadedRender (lit skeleton core).
//!
//! Walks a bone, looks up its material, and evaluates the light
//! rig. No unwraps.
//!
//! Differential: a bone with a red material under intensity 1
//! returns a red lit color.

use crate::bone::Bone;
use crate::light_rig::LightRig;
use crate::lighting::Lighting;
use crate::material_assign::MaterialAssign;
use crate::material_registry::MaterialRegistry;
use crate::vec3::Vec3;

#[derive(Clone, Debug)]
pub struct ShadedItem {
    pub bone: String,
    pub color: Vec3,
}

pub struct ShadedRender;

impl ShadedRender {
    /// Shade a single bone. Missing material falls back to gray.
    pub fn shade_bone(
        bone: &Bone,
        assign: &MaterialAssign,
        registry: &MaterialRegistry,
        rig: &LightRig,
    ) -> ShadedItem {
        let name = bone.name().to_string();
        let color = if let Ok(mat_name) = assign.material_of(bone.name()) {
            if let Ok(m) = registry.get(mat_name) {
                Lighting::evaluate(m, rig)
            } else {
                Vec3::new(0.5, 0.5, 0.5)
            }
        } else {
            Vec3::new(0.5, 0.5, 0.5)
        };
        ShadedItem { bone: name, color }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn shades_red() {
        let bone = Bone::new("plane").unwrap();
        let mut assign = MaterialAssign::new();
        assign.assign("plane", "paint").unwrap();
        let mut registry = MaterialRegistry::new();
        registry.add("paint").unwrap();
        registry.get_mut("paint").unwrap().set_base_color(crate::vec3::Vec3::new(1.0, 0.0, 0.0));
        registry.get_mut("paint").unwrap().set_roughness(0.0);
        let mut rig = LightRig::new();
        rig.add("key").unwrap();
        let item = ShadedRender::shade_bone(&bone, &assign, &registry, &rig);
        assert_eq!(item.bone, "plane");
        assert!((item.color.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let bone = Bone::new("wing").unwrap();
        let assign = MaterialAssign::new();
        let registry = MaterialRegistry::new();
        let rig = LightRig::new();
        let item = ShadedRender::shade_bone(&bone, &assign, &registry, &rig);
        assert_eq!(item.color.x, 0.5);
    }
}
