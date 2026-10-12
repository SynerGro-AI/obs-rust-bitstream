//! Safe pure MaterialFactory (preset material creator).
//!
//! Creates common materials by name and stores them in a
//! MaterialRegistry. Unknown names fall back to a neutral gray.
//! No unwraps.
//!
//! Differential: create "chrome" + get roughness round-trips.

use crate::material_registry::{MaterialRegistry, MaterialRegistryError};
use crate::vec3::Vec3;

pub struct MaterialFactory;

impl MaterialFactory {
    /// Create a preset material and add it to the registry.
    /// Known presets: chrome, paint, glass, rubber, gold.
    pub fn create(registry: &mut MaterialRegistry, name: &str) -> Result<(), MaterialRegistryError> {
        registry.add(name)?;
        let m = registry.get_mut(name)?;
        match name {
            "chrome" => {
                m.set_base_color(Vec3::new(0.8, 0.8, 0.85));
                m.set_roughness(0.05);
                m.set_metallic(1.0);
            }
            "paint" => {
                m.set_base_color(Vec3::new(0.2, 0.4, 0.8));
                m.set_roughness(0.4);
                m.set_metallic(0.0);
            }
            "glass" => {
                m.set_base_color(Vec3::new(0.9, 0.95, 1.0));
                m.set_roughness(0.0);
                m.set_metallic(0.0);
            }
            "rubber" => {
                m.set_base_color(Vec3::new(0.1, 0.1, 0.1));
                m.set_roughness(0.9);
                m.set_metallic(0.0);
            }
            "gold" => {
                m.set_base_color(Vec3::new(1.0, 0.84, 0.0));
                m.set_roughness(0.2);
                m.set_metallic(1.0);
            }
            _ => {
                m.set_base_color(Vec3::new(0.5, 0.5, 0.5));
                m.set_roughness(0.5);
                m.set_metallic(0.0);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_chrome() {
        let mut r = MaterialRegistry::new();
        MaterialFactory::create(&mut r, "chrome").unwrap();
        let m = r.get("chrome").unwrap();
        assert_eq!(m.metallic(), 1.0);
        assert!(m.roughness() < 0.1);
    }

    #[test]
    fn differential() {
        let mut r = MaterialRegistry::new();
        MaterialFactory::create(&mut r, "gold").unwrap();
        let m = r.get("gold").unwrap();
        assert_eq!(m.base_color().y, 0.84);
        assert_eq!(m.metallic(), 1.0);
    }

    #[test]
    fn unknown_is_gray() {
        let mut r = MaterialRegistry::new();
        MaterialFactory::create(&mut r, "custom").unwrap();
        let m = r.get("custom").unwrap();
        assert_eq!(m.base_color().x, 0.5);
    }
}
