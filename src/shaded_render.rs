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
use crate::material::Material;
use crate::material_assign::MaterialAssign;
use crate::vec3::Vec3;
use std::collections::HashMap;

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
        materials: &HashMap<String, Material>,
        rig: &LightRig,
    ) -> ShadedItem {
        let name = bone.name().to_string();
        let color = if let Ok(mat_name) = assign.material_of(bone.name()) {
            if let Some(m) = materials.get(mat_name) {
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
        let mut materials = HashMap::new();
        let mut m = Material::new("paint").unwrap();
        m.set_base_color(Vec3::new(1.0, 0.0, 0.0));
        m.set_roughness(0.0);
        materials.insert("paint".to_string(), m);
        let mut rig = LightRig::new();
        rig.add("key").unwrap();
        let item = ShadedRender::shade_bone(&bone, &assign, &materials, &rig);
        assert_eq!(item.bone, "plane");
        assert!((item.color.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let bone = Bone::new("wing").unwrap();
        let assign = MaterialAssign::new();
        let materials = HashMap::new();
        let rig = LightRig::new();
        let item = ShadedRender::shade_bone(&bone, &assign, &materials, &rig);
        assert_eq!(item.color.x, 0.5);
    }
}
