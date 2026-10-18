//! Safe pure SourceList (ordered capture sources).
//!
//! Holds named sources and returns the active one. No unwraps.
//!
//! Differential: an added source can be found by name.

use crate::capture_select::CaptureSource;

pub struct SourceList {
    items: Vec<(String, CaptureSource)>,
    active: usize,
}

impl SourceList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
        }
    }

    /// Add a source. Returns its index.
    pub fn add(&mut self, name: &str, src: CaptureSource) -> usize {
        self.items.push((name.to_string(), src));
        self.items.len() - 1
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Names in order.
    pub fn names(&self) -> Vec<&str> {
        self.items.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Set the active index. Clamped.
    pub fn set_active(&mut self, i: usize) {
        if !self.items.is_empty() {
            self.active = i.min(self.items.len() - 1);
        }
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    /// Active source, if any.
    pub fn active(&mut self) -> Option<&mut CaptureSource> {
        self.items.get_mut(self.active).map(|(_, s)| s)
    }

    /// Find by name.
    pub fn find(&mut self, name: &str) -> Option<&mut CaptureSource> {
        self.items.iter_mut().find(|(n, _)| n == name).map(|(_, s)| s)
    }
}

impl Default for SourceList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut list = SourceList::new();
        list.add("a", CaptureSource::synthetic(4, 4));
        list.add("b", CaptureSource::synthetic(8, 8));
        assert_eq!(list.len(), 2);
        assert_eq!(list.names(), vec!["a", "b"]);
        assert!(list.find("b").is_some());
        list.set_active(1);
        assert_eq!(list.active_index(), 1);
    }
}
