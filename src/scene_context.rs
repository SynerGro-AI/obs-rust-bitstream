//! Safe pure SceneContext (OBS scene core).
//!
//! A scene owns a name and an ordered list of SceneItems.
//! Empty names are rejected. Duplicate item ids are rejected.
//! Items carry transform and visibility. No unwraps.
//!
//! Differential: add item + get transform round-trips.

use crate::scene_item::{SceneItem, SceneItemError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    EmptyName,
    DuplicateItem,
    MissingItem,
    EmptySource,
}

impl From<SceneItemError> for SceneError {
    fn from(e: SceneItemError) -> Self {
        match e {
            SceneItemError::EmptyId => SceneError::EmptyName,
            SceneItemError::EmptySource => SceneError::EmptySource,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneContext {
    name: String,
    items: Vec<SceneItem>,
}

impl SceneContext {
    pub fn new(name: &str) -> Result<Self, SceneError> {
        if name.is_empty() {
            return Err(SceneError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            items: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add an item by id and source name. Empty and duplicate ids are rejected.
    pub fn add_item(&mut self, id: &str, source: &str) -> Result<(), SceneError> {
        if self.items.iter().any(|i| i.id() == id) {
            return Err(SceneError::DuplicateItem);
        }
        let item = SceneItem::new(id, source)?;
        self.items.push(item);
        Ok(())
    }

    /// Remove an item by id. Returns true if it was present.
    pub fn remove_item(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id() != id);
        self.items.len() != before
    }

    pub fn get(&self, id: &str) -> Option<&SceneItem> {
        self.items.iter().find(|i| i.id() == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut SceneItem> {
        self.items.iter_mut().find(|i| i.id() == id)
    }

    pub fn items(&self) -> &[SceneItem] {
        &self.items
    }

    pub fn ids(&self) -> Vec<&str> {
        self.items.iter().map(|i| i.id()).collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_add() {
        let mut s = SceneContext::new("Main").unwrap();
        s.add_item("cam", "camera_src").unwrap();
        s.add_item("overlay", "overlay_src").unwrap();
        assert_eq!(s.ids(), vec!["cam", "overlay"]);
    }

    #[test]
    fn empty_and_duplicate() {
        assert!(SceneContext::new("").is_err());
        let mut s = SceneContext::new("s").unwrap();
        s.add_item("a", "src").unwrap();
        assert!(matches!(s.add_item("a", "src"), Err(SceneError::DuplicateItem)));
        assert!(matches!(s.add_item("", "src"), Err(SceneError::EmptyName)));
        assert!(matches!(s.add_item("b", ""), Err(SceneError::EmptySource)));
    }

    #[test]
    fn remove() {
        let mut s = SceneContext::new("s").unwrap();
        s.add_item("a", "src").unwrap();
        assert!(s.remove_item("a"));
        assert!(!s.contains("a"));
        assert!(!s.remove_item("a"));
    }

    #[test]
    fn differential() {
        let mut s = SceneContext::new("scene").unwrap();
        s.add_item("src1", "cam").unwrap();
        s.get_mut("src1").unwrap().set_pos(50.0, 60.0);
        assert_eq!(s.get("src1").unwrap().transform().pos.x, 50.0);
        assert!(s.contains("src1"));
    }
}
