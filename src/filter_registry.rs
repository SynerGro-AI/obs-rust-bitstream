//! Safe pure FilterRegistry (OBS filter type lookup core).
//!
//! Name-to-FilterContext map for registered filter types.
//! Empty and duplicate names are rejected. No unwraps.
//!
//! Differential: add + get returns the same filter; settings round-trip.

use std::collections::HashMap;

use crate::filter_context::{FilterContext, FilterError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<FilterError> for FilterRegistryError {
    fn from(_: FilterError) -> Self {
        FilterRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterRegistry {
    filters: HashMap<String, FilterContext>,
}

impl FilterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, id: &str) -> Result<(), FilterRegistryError> {
        if name.is_empty() {
            return Err(FilterRegistryError::EmptyName);
        }
        if self.filters.contains_key(name) {
            return Err(FilterRegistryError::Duplicate);
        }
        let ctx = FilterContext::new(name, id)?;
        self.filters.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&FilterContext, FilterRegistryError> {
        self.filters.get(name).ok_or(FilterRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut FilterContext, FilterRegistryError> {
        self.filters.get_mut(name).ok_or(FilterRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.filters.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.filters.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.filters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = FilterRegistry::new();
        r.add("blur", "gpu_blur").unwrap();
        assert_eq!(r.get("blur").unwrap().id(), "gpu_blur");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = FilterRegistry::new();
        r.add("a", "id").unwrap();
        assert!(matches!(r.add("a", "id"), Err(FilterRegistryError::Duplicate)));
        assert!(matches!(r.add("", "id"), Err(FilterRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = FilterRegistry::new();
        assert!(matches!(r.get("nope"), Err(FilterRegistryError::Missing)));
        r.add("f", "id").unwrap();
        assert!(r.remove("f"));
        assert!(!r.contains("f"));
    }

    #[test]
    fn differential() {
        let mut r = FilterRegistry::new();
        r.add("gain", "gain_filter").unwrap();
        r.get_mut("gain").unwrap().settings_mut().set_double("db", -6.0).unwrap();
        assert_eq!(r.get("gain").unwrap().settings().get_double("db").unwrap(), -6.0);
    }
}
