//! Safe pure SceneSwitch (scene switch core).
//!
//! Holds the current scene name. switch changes it.
//! Empty names rejected. No unwraps.
//!
//! Differential: switch + current round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchError {
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct SceneSwitch {
    current: Option<String>,
}

impl SceneSwitch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn switch(&mut self, name: &str) -> Result<(), SwitchError> {
        if name.is_empty() {
            return Err(SwitchError::EmptyName);
        }
        self.current = Some(name.to_string());
        Ok(())
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let s = SceneSwitch::new();
        assert!(s.current().is_none());
    }

    #[test]
    fn empty_name() {
        let mut s = SceneSwitch::new();
        assert!(s.switch("").is_err());
    }

    #[test]
    fn differential() {
        let mut s = SceneSwitch::new();
        s.switch("wide").unwrap();
        assert_eq!(s.current(), Some("wide"));
        s.switch("close").unwrap();
        assert_eq!(s.current(), Some("close"));
    }
}
