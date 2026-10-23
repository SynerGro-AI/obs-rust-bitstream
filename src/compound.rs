//! Safe pure Compound (molecule core).
//!
//! Binds a list of atoms into a named compound and synthesizes
//! a material from the average binding. Empty names and empty
//! compositions are rejected. No unwraps.
//!
//! Differential: H2O (2H + 1O) produces a material.

use crate::material_registry::{MaterialRegistry, MaterialRegistryError};
use crate::materials_lab::Atom;
use crate::vec3::Vec3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompoundError {
    EmptyName,
    EmptyComposition,
    Unstable,
    Registry(MaterialRegistryError),
}

impl From<MaterialRegistryError> for CompoundError {
    fn from(e: MaterialRegistryError) -> Self {
        match e {
            MaterialRegistryError::EmptyName => CompoundError::EmptyName,
            other => CompoundError::Registry(other),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Compound {
    name: String,
    atoms: Vec<Atom>,
}

impl Compound {
    pub fn new(name: &str) -> Result<Self, CompoundError> {
        if name.is_empty() {
            return Err(CompoundError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            atoms: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, atom: Atom) {
        self.atoms.push(atom);
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    /// True if every atom is stable.
    pub fn is_stable(&self) -> bool {
        !self.atoms.is_empty() && self.atoms.iter().all(|a| a.is_stable())
    }

    /// Average binding energy per nucleon across the compound.
    pub fn average_binding(&self) -> f32 {
        if self.atoms.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.atoms.iter().map(|a| a.binding_per_nucleon()).sum();
        sum / self.atoms.len() as f32
    }

    /// Synthesize a material from the compound.
    pub fn synthesize(&self, registry: &mut MaterialRegistry) -> Result<(), CompoundError> {
        if self.atoms.is_empty() {
            return Err(CompoundError::EmptyComposition);
        }
        if !self.is_stable() {
            return Err(CompoundError::Unstable);
        }
        registry.add(&self.name)?;
        let m = registry.get_mut(&self.name)?;
        let be = self.average_binding();
        let metallic = (be / 9.0).clamp(0.0, 1.0);
        m.set_metallic(metallic);
        m.set_roughness((1.0 - metallic).clamp(0.0, 1.0));
        m.set_base_color(Vec3::new(0.7, 0.8, 0.9));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water() {
        let mut c = Compound::new("water").unwrap();
        c.add(Atom::new(1, 0, 1));
        c.add(Atom::new(1, 0, 1));
        c.add(Atom::new(8, 8, 8));
        assert!(c.is_stable());
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn empty_name() {
        assert!(Compound::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut c = Compound::new("water").unwrap();
        c.add(Atom::new(1, 0, 1));
        c.add(Atom::new(8, 8, 8));
        let mut r = MaterialRegistry::new();
        c.synthesize(&mut r).unwrap();
        assert!(r.get("water").is_ok());
    }
}
