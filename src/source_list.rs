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

    /// True if a name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.items.iter().any(|(n, _)| n == name)
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

    /// Name of the active source, if any.
    pub fn active_name(&self) -> Option<&str> {
        self.items.get(self.active).map(|(n, _)| n.as_str())
    }

    /// Switch to the source named. Returns true if found.
    pub fn switch(&mut self, name: &str) -> bool {
        if let Some(i) = self.items.iter().position(|(n, _)| n == name) {
            self.active = i;
            true
        } else {
            false
        }
    }

    /// Remove by name. Returns true if found.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(i) = self.items.iter().position(|(n, _)| n == name) {
            self.items.remove(i);
            if self.active >= self.items.len() && !self.items.is_empty() {
                self.active = self.items.len() - 1;
            }
            if self.items.is_empty() {
                self.active = 0;
            }
            true
        } else {
            false
        }
    }

    /// Remove all sources.
    pub fn clear(&mut self) {
        self.items.clear();
        self.active = 0;
    }

    /// Rename a source. Returns true if found.
    pub fn rename(&mut self, from: &str, to: &str) -> bool {
        if let Some(i) = self.items.iter().position(|(n, _)| n == from) {
            self.items[i].0 = to.to_string();
            true
        } else {
            false
        }
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
        assert_eq!(list.active_name(), Some("b"));
        assert!(list.switch("a"));
        assert_eq!(list.active_index(), 0);
        assert!(!list.switch("missing"));
        assert!(list.remove("b"));
        assert_eq!(list.len(), 1);
        assert!(!list.remove("missing"));
        let mut list2 = SourceList::new();
        list2.add("old", CaptureSource::synthetic(4, 4));
        assert!(list2.rename("old", "new"));
        assert_eq!(list2.names(), vec!["new"]);
        assert!(list2.contains("new"));
        assert!(!list2.contains("old"));
    }
}
