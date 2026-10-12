//! Safe pure SkeletonConnector (AI bone wiring core).
//!
//! Holds a root bone and can ask an AiAssistant which child
//! should attach to which parent, then wires the link.
//! Empty names are rejected. No unwraps.
//!
//! Differential: connect a child + render count round-trips.

use crate::ai::AiAssistant;
use crate::bone::{Bone, BoneError};
use crate::character_renderer::CharacterRenderer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectorError {
    EmptyName,
    Missing,
}

impl From<BoneError> for ConnectorError {
    fn from(_: BoneError) -> Self {
        ConnectorError::EmptyName
    }
}

#[derive(Clone, Debug)]
pub struct SkeletonConnector {
    root: Bone,
}

impl SkeletonConnector {
    pub fn new(root_name: &str) -> Result<Self, ConnectorError> {
        Ok(Self {
            root: Bone::new(root_name)?,
        })
    }

    pub fn root(&self) -> &Bone {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut Bone {
        &mut self.root
    }

    /// Attach a child bone directly to the root.
    pub fn connect(&mut self, child_name: &str) -> Result<(), ConnectorError> {
        let child = Bone::new(child_name)?;
        self.root.add_child(child);
        Ok(())
    }

    /// Ask the assistant for a child name and connect it.
    /// Falls back to `fallback` if the reply has no usable token.
    pub fn connect_ai(
        &mut self,
        ai: &AiAssistant,
        prompt: &str,
        fallback: &str,
    ) -> Result<(), ConnectorError> {
        let reply = ai.assist(prompt);
        let name = first_token(&reply).unwrap_or(fallback);
        if name.is_empty() {
            return Err(ConnectorError::EmptyName);
        }
        self.connect(name)
    }

    pub fn child_count(&self) -> usize {
        self.root.children().len()
    }

    /// Render the connected skeleton.
    pub fn render(&self) -> Vec<crate::render_item::RenderItem> {
        CharacterRenderer::new().render(&self.root)
    }
}

fn first_token(s: &str) -> Option<&str> {
    s.split_whitespace()
        .find(|t| !t.starts_with('[') && !t.contains('=') && !t.contains('|'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect() {
        let mut c = SkeletonConnector::new("root").unwrap();
        c.connect("spine").unwrap();
        assert_eq!(c.child_count(), 1);
        assert_eq!(c.render().len(), 2);
    }

    #[test]
    fn empty_name() {
        assert!(SkeletonConnector::new("").is_err());
        let mut c = SkeletonConnector::new("root").unwrap();
        assert!(c.connect("").is_err());
    }

    #[test]
    fn ai_connect() {
        let mut c = SkeletonConnector::new("root").unwrap();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        c.connect_ai(&ai, "suggest a child bone", "arm").unwrap();
        assert_eq!(c.child_count(), 1);
    }

    #[test]
    fn differential() {
        let mut c = SkeletonConnector::new("root").unwrap();
        c.connect("spine").unwrap();
        c.connect("arm").unwrap();
        assert_eq!(c.render().len(), 3);
    }
}
