//! Safe pure Properties (OBS obs-properties.h core).
//!
//! The C version is an ordered list of named settings fields
//! (bool, int, float, text, list, ...). The safe path is a Vec of
//! typed properties. Duplicate names and empty names are rejected.
//! No unwraps.
//!
//! Differential: add + get returns the same property.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    Text,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub name: String,
    pub description: String,
    pub ty: PropertyType,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropsError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct Properties {
    items: Vec<Property>,
}

impl Properties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bool(&mut self, name: &str, description: &str) -> Result<(), PropsError> {
        self.add(Property {
            name: name.to_string(),
            description: description.to_string(),
            ty: PropertyType::Bool,
            min: None,
            max: None,
            step: None,
        })
    }

    pub fn add_int(
        &mut self,
        name: &str,
        description: &str,
        min: i64,
        max: i64,
        step: i64,
    ) -> Result<(), PropsError> {
        self.add(Property {
            name: name.to_string(),
            description: description.to_string(),
            ty: PropertyType::Int,
            min: Some(min as f64),
            max: Some(max as f64),
            step: Some(step as f64),
        })
    }

    pub fn add_float(
        &mut self,
        name: &str,
        description: &str,
        min: f64,
        max: f64,
        step: f64,
    ) -> Result<(), PropsError> {
        self.add(Property {
            name: name.to_string(),
            description: description.to_string(),
            ty: PropertyType::Float,
            min: Some(min),
            max: Some(max),
            step: Some(step),
        })
    }

    pub fn add_text(&mut self, name: &str, description: &str) -> Result<(), PropsError> {
        self.add(Property {
            name: name.to_string(),
            description: description.to_string(),
            ty: PropertyType::Text,
            min: None,
            max: None,
            step: None,
        })
    }

    fn add(&mut self, prop: Property) -> Result<(), PropsError> {
        if prop.name.is_empty() {
            return Err(PropsError::EmptyName);
        }
        if self.items.iter().any(|p| p.name == prop.name) {
            return Err(PropsError::Duplicate);
        }
        self.items.push(prop);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Property, PropsError> {
        self.items
            .iter()
            .find(|p| p.name == name)
            .ok_or(PropsError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|p| p.name != name);
        self.items.len() != before
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.items.iter().map(|p| p.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut p = Properties::new();
        p.add_bool("visible", "Visible").unwrap();
        p.add_int("width", "Width", 1, 4096, 1).unwrap();
        p.add_text("name", "Name").unwrap();
        assert_eq!(p.get("visible").unwrap().ty, PropertyType::Bool);
        assert_eq!(p.get("width").unwrap().min, Some(1.0));
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut p = Properties::new();
        p.add_bool("a", "A").unwrap();
        assert_eq!(p.add_bool("a", "A2"), Err(PropsError::Duplicate));
        assert_eq!(p.add_text("", "x"), Err(PropsError::EmptyName));
    }

    #[test]
    fn missing() {
        let p = Properties::new();
        assert_eq!(p.get("nope"), Err(PropsError::Missing));
    }

    #[test]
    fn differential() {
        let mut p = Properties::new();
        p.add_float("gain", "Gain", 0.0, 1.0, 0.01).unwrap();
        let g = p.get("gain").unwrap();
        assert_eq!(g.ty, PropertyType::Float);
        assert_eq!(g.max, Some(1.0));
    }
}
