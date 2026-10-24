//! Safe pure History (undo / redo core).
//!
//! Records named edit steps. undo pops the last step.
//! redo restores it. No unwraps.
//!
//! Differential: push two steps + undo + redo round-trips.

#[derive(Clone, Debug)]
pub struct History {
    past: Vec<String>,
    future: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
        }
    }

    pub fn push(&mut self, step: &str) {
        self.past.push(step.to_string());
        self.future.clear();
    }

    pub fn undo(&mut self) -> Option<String> {
        let step = self.past.pop()?;
        self.future.push(step.clone());
        Some(step)
    }

    pub fn redo(&mut self) -> Option<String> {
        let step = self.future.pop()?;
        self.past.push(step.clone());
        Some(step)
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn len(&self) -> usize {
        self.past.len()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let h = History::new();
        assert!(!h.can_undo());
        assert!(!h.can_redo());
    }

    #[test]
    fn differential() {
        let mut h = History::new();
        h.push("add layer");
        h.push("set blend");
        assert_eq!(h.undo(), Some("set blend".to_string()));
        assert!(h.can_redo());
        assert_eq!(h.redo(), Some("set blend".to_string()));
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn push_clears_future() {
        let mut h = History::new();
        h.push("a");
        h.undo();
        h.push("b");
        assert!(!h.can_redo());
    }
}
