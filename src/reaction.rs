//! Safe pure Reaction (chemical reaction core).
//!
//! Takes reactant compounds and produces a named product
//! material from their combined binding. Empty names and
//! empty reactant lists are rejected. No unwraps.
//!
//! Differential: H2 + O produces water.

use crate::compound::Compound;
use crate::material_registry::{MaterialRegistry, MaterialRegistryError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReactionError {
    EmptyName,
    EmptyReactants,
    Unstable,
    Registry(MaterialRegistryError),
}

impl From<MaterialRegistryError> for ReactionError {
    fn from(e: MaterialRegistryError) -> Self {
        match e {
            MaterialRegistryError::EmptyName => ReactionError::EmptyName,
            other => ReactionError::Registry(other),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reaction {
    product: String,
    reactants: Vec<Compound>,
}

impl Reaction {
    pub fn new(product: &str) -> Result<Self, ReactionError> {
        if product.is_empty() {
            return Err(ReactionError::EmptyName);
        }
        Ok(Self {
            product: product.to_string(),
            reactants: Vec::new(),
        })
    }

    pub fn product(&self) -> &str {
        &self.product
    }

    pub fn add_reactant(&mut self, c: Compound) {
        self.reactants.push(c);
    }

    pub fn reactant_count(&self) -> usize {
        self.reactants.len()
    }

    /// Run the reaction. All reactants must be stable.
    pub fn run(&self, registry: &mut MaterialRegistry) -> Result<(), ReactionError> {
        if self.reactants.is_empty() {
            return Err(ReactionError::EmptyReactants);
        }
        if !self.reactants.iter().all(|c| c.is_stable()) {
            return Err(ReactionError::Unstable);
        }
        let avg: f32 = self
            .reactants
            .iter()
            .map(|c| c.average_binding())
            .sum::<f32>()
            / self.reactants.len() as f32;
        registry.add(&self.product)?;
        let m = registry.get_mut(&self.product)?;
        let metallic = (avg / 9.0).clamp(0.0, 1.0);
        m.set_metallic(metallic);
        m.set_roughness((1.0 - metallic).clamp(0.0, 1.0));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials_lab::Atom;

    #[test]
    fn water_reaction() {
        let mut h2 = Compound::new("H2").unwrap();
        h2.add(Atom::new(1, 0, 1));
        h2.add(Atom::new(1, 0, 1));
        let mut o = Compound::new("O").unwrap();
        o.add(Atom::new(8, 8, 8));
        let mut rxn = Reaction::new("water").unwrap();
        rxn.add_reactant(h2);
        rxn.add_reactant(o);
        let mut r = MaterialRegistry::new();
        rxn.run(&mut r).unwrap();
        assert!(r.get("water").is_ok());
    }

    #[test]
    fn empty_name() {
        assert!(Reaction::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut h = Compound::new("H").unwrap();
        h.add(Atom::new(1, 0, 1));
        let mut rxn = Reaction::new("product").unwrap();
        rxn.add_reactant(h);
        assert_eq!(rxn.reactant_count(), 1);
        let mut r = MaterialRegistry::new();
        rxn.run(&mut r).unwrap();
        assert!(r.get("product").is_ok());
    }
}
