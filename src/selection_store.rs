//! Safe pure SelectionStore (save/load selection core).
//!
//! Holds named selections. Empty names rejected.
//! No unwraps.
//!
//! Differential: save + load round-trips.

use crate::pixel_selection::Selection;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct SelectionStore {
    saved: BTreeMap<String, Selection>,
}

impl SelectionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self, name: &str, sel: &Selection) -> Result<(), StoreError> {
        if name.is_empty() {
            return Err(StoreError::EmptyName);
        }
        self.saved.insert(name.to_string(), sel.clone());
        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<Selection, StoreError> {
        self.saved.get(name).cloned().ok_or(StoreError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.saved.remove(name).is_some()
    }

    pub fn names(&self) -> Vec<&str> {
        self.saved.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.saved.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut store = SelectionStore::new();
        let s = Selection::new(2, 2);
        assert!(store.save("", &s).is_err());
        assert!(store.load("missing").is_err());
    }

    #[test]
    fn differential() {
        let mut store = SelectionStore::new();
        let mut s = Selection::new(4, 4);
        s.rect(1, 1, 2, 2).unwrap();
        store.save("box", &s).unwrap();
        let loaded = store.load("box").unwrap();
        assert!(loaded.is_selected(1, 1));
        assert_eq!(loaded.count(), s.count());
    }
}
