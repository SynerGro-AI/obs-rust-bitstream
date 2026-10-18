//! Safe pure SceneList (ordered scenes).
//!
//! Holds named scenes and returns the active one. No unwraps.
//!
//! Differential: an added scene can be found by name.

use crate::scene::Scene;

pub struct SceneList {
    items: Vec<Scene>,
    active: usize,
}

impl SceneList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
        }
    }

    /// Add a scene. Returns its index.
    pub fn add(&mut self, scene: Scene) -> usize {
        self.items.push(scene);
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
        self.items.iter().map(|s| s.name()).collect()
    }

    /// Summaries in order.
    pub fn summaries(&self) -> Vec<String> {
        self.items.iter().map(|s| s.summary()).collect()
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

    /// Name of the active scene, if any.
    pub fn active_name(&self) -> Option<&str> {
        self.items.get(self.active).map(|s| s.name())
    }

    /// Switch to the scene named. Returns true if found.
    pub fn switch(&mut self, name: &str) -> bool {
        if let Some(i) = self.items.iter().position(|s| s.name() == name) {
            self.active = i;
            true
        } else {
            false
        }
    }

    /// Remove by name. Returns true if found.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(i) = self.items.iter().position(|s| s.name() == name) {
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

    /// Remove all scenes.
    pub fn clear(&mut self) {
        self.items.clear();
        self.active = 0;
    }

    /// Active scene, if any.
    pub fn active(&mut self) -> Option<&mut Scene> {
        self.items.get_mut(self.active)
    }

    /// Find by name.
    pub fn find(&mut self, name: &str) -> Option<&mut Scene> {
        self.items.iter_mut().find(|s| s.name() == name)
    }
}

impl Default for SceneList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut list = SceneList::new();
        list.add(Scene::new("a"));
        list.add(Scene::new("b"));
        assert_eq!(list.len(), 2);
        assert_eq!(list.names(), vec!["a", "b"]);
        assert!(list.find("b").is_some());
        list.set_active(1);
        assert_eq!(list.active_index(), 1);
        assert!(list.switch("a"));
        assert_eq!(list.active_index(), 0);
        assert!(!list.switch("missing"));
        assert!(list.remove("b"));
        assert_eq!(list.len(), 1);
        list.clear();
        assert!(list.is_empty());
    }
}
