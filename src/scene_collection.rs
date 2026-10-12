//! Safe pure SceneCollection (OBS scene collection core).
//!
//! A collection owns a name and a list of scene names. Empty
//! names are rejected. Duplicate scene names are rejected.
//! No unwraps.
//!
//! Differential: add scene + contains round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionError {
    EmptyName,
    Duplicate,
}

#[derive(Clone, Debug)]
pub struct SceneCollection {
    name: String,
    scenes: Vec<String>,
}

impl SceneCollection {
    pub fn new(name: &str) -> Result<Self, CollectionError> {
        if name.is_empty() {
            return Err(CollectionError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            scenes: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_scene(&mut self, name: &str) -> Result<(), CollectionError> {
        if name.is_empty() {
            return Err(CollectionError::EmptyName);
        }
        if self.scenes.iter().any(|s| s == name) {
            return Err(CollectionError::Duplicate);
        }
        self.scenes.push(name.to_string());
        Ok(())
    }

    pub fn remove_scene(&mut self, name: &str) -> bool {
        let before = self.scenes.len();
        self.scenes.retain(|s| s != name);
        self.scenes.len() != before
    }

    pub fn contains(&self, name: &str) -> bool {
        self.scenes.iter().any(|s| s == name)
    }

    pub fn scenes(&self) -> &[String] {
        &self.scenes
    }

    pub fn len(&self) -> usize {
        self.scenes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_add() {
        let mut c = SceneCollection::new("Default").unwrap();
        c.add_scene("Main").unwrap();
        c.add_scene("Overlay").unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.contains("Main"));
    }

    #[test]
    fn empty_and_duplicate() {
        assert!(SceneCollection::new("").is_err());
        let mut c = SceneCollection::new("c").unwrap();
        c.add_scene("a").unwrap();
        assert!(matches!(c.add_scene("a"), Err(CollectionError::Duplicate)));
        assert!(matches!(c.add_scene(""), Err(CollectionError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut c = SceneCollection::new("col").unwrap();
        c.add_scene("Main").unwrap();
        assert!(c.contains("Main"));
        assert!(c.remove_scene("Main"));
        assert!(!c.contains("Main"));
    }
}
