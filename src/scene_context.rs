//! Safe pure SceneContext (OBS scene core).
//!
//! A scene owns a name and an ordered list of item ids.
//! Empty names are rejected. Duplicate item ids are rejected.
//! No unwraps.
//!
//! Differential: add item + get order round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    EmptyName,
    DuplicateItem,
    MissingItem,
}

#[derive(Clone, Debug)]
pub struct SceneContext {
    name: String,
    items: Vec<String>,
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

    /// Add an item id at the end. Empty and duplicate ids are rejected.
    pub fn add_item(&mut self, id: &str) -> Result<(), SceneError> {
        if id.is_empty() {
            return Err(SceneError::EmptyName);
        }
        if self.items.iter().any(|i| i == id) {
            return Err(SceneError::DuplicateItem);
        }
        self.items.push(id.to_string());
        Ok(())
    }

    /// Remove an item by id. Returns true if it was present.
    pub fn remove_item(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i != id);
        self.items.len() != before
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|i| i == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_add() {
        let mut s = SceneContext::new("Main").unwrap();
        s.add_item("cam").unwrap();
        s.add_item("overlay").unwrap();
        assert_eq!(s.items(), &["cam".to_string(), "overlay".to_string()]);
    }

    #[test]
    fn empty_and_duplicate() {
        assert!(SceneContext::new("").is_err());
        let mut s = SceneContext::new("s").unwrap();
        s.add_item("a").unwrap();
        assert!(matches!(s.add_item("a"), Err(SceneError::DuplicateItem)));
        assert!(matches!(s.add_item(""), Err(SceneError::EmptyName)));
    }

    #[test]
    fn remove() {
        let mut s = SceneContext::new("s").unwrap();
        s.add_item("a").unwrap();
        assert!(s.remove_item("a"));
        assert!(!s.contains("a"));
        assert!(!s.remove_item("a"));
    }

    #[test]
    fn differential() {
        let mut s = SceneContext::new("scene").unwrap();
        s.add_item("src1").unwrap();
        assert_eq!(s.len(), 1);
        assert!(s.contains("src1"));
    }
}
