//! Safe pure FilterChain (OBS source filter list).
//!
//! Ordered list of FilterContext. Duplicate names and empty
//! names are rejected. No unwraps.
//!
//! Differential: add + get returns the same filter; settings round-trip.

use crate::filter_context::{FilterContext, FilterError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterChainError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<FilterError> for FilterChainError {
    fn from(_: FilterError) -> Self {
        FilterChainError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterChain {
    filters: Vec<FilterContext>,
}

impl FilterChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter at the end. Empty and duplicate names are rejected.
    pub fn add(&mut self, name: &str, id: &str) -> Result<(), FilterChainError> {
        if self.filters.iter().any(|f| f.name() == name) {
            return Err(FilterChainError::Duplicate);
        }
        let f = FilterContext::new(name, id)?;
        self.filters.push(f);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&FilterContext, FilterChainError> {
        self.filters
            .iter()
            .find(|f| f.name() == name)
            .ok_or(FilterChainError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut FilterContext, FilterChainError> {
        self.filters
            .iter_mut()
            .find(|f| f.name() == name)
            .ok_or(FilterChainError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.filters.len();
        self.filters.retain(|f| f.name() != name);
        self.filters.len() != before
    }

    pub fn len(&self) -> usize {
        self.filters.len()
    }

    pub fn names(&self) -> Vec<&str> {
        self.filters.iter().map(|f| f.name()).collect()
    }

    /// Move a filter up (toward the start). Returns false if already first or missing.
    pub fn move_up(&mut self, name: &str) -> bool {
        if let Some(i) = self.filters.iter().position(|f| f.name() == name) {
            if i > 0 {
                self.filters.swap(i, i - 1);
                return true;
            }
        }
        false
    }

    /// Move a filter down (toward the end). Returns false if already last or missing.
    pub fn move_down(&mut self, name: &str) -> bool {
        if let Some(i) = self.filters.iter().position(|f| f.name() == name) {
            if i + 1 < self.filters.len() {
                self.filters.swap(i, i + 1);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut c = FilterChain::new();
        c.add("blur", "gpu_blur").unwrap();
        c.add("gain", "gain_filter").unwrap();
        assert_eq!(c.names(), vec!["blur", "gain"]);
        assert_eq!(c.get("blur").unwrap().id(), "gpu_blur");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut c = FilterChain::new();
        c.add("a", "id").unwrap();
        assert!(matches!(c.add("a", "id"), Err(FilterChainError::Duplicate)));
        assert!(matches!(c.add("", "id"), Err(FilterChainError::EmptyName)));
    }

    #[test]
    fn reorder() {
        let mut c = FilterChain::new();
        c.add("a", "id").unwrap();
        c.add("b", "id").unwrap();
        assert!(c.move_up("b"));
        assert_eq!(c.names(), vec!["b", "a"]);
        assert!(c.move_down("b"));
        assert_eq!(c.names(), vec!["a", "b"]);
    }

    #[test]
    fn differential() {
        let mut c = FilterChain::new();
        c.add("gain", "gain_filter").unwrap();
        c.get_mut("gain")
            .unwrap()
            .settings_mut()
            .set_double("db", -3.0)
            .unwrap();
        assert_eq!(
            c.get("gain").unwrap().settings().get_double("db").unwrap(),
            -3.0
        );
    }
}
