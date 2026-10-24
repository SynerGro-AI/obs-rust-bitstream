//! Safe pure CharacterRenderer (digital character core).
//!
//! Walks a bone hierarchy, connects every child, and builds a
//! render list of the skeleton. An AiAssistant can suggest a
//! character look. No unwraps.
//!
//! Differential: a root with one child produces two render items.

use crate::ai::AiAssistant;
use crate::bone::Bone;
use crate::matrix4::Matrix4;
use crate::render_item::RenderItem;
use crate::scene_item::Transform;
use crate::vec2::Vec2;

#[derive(Clone, Debug)]
pub struct CharacterRenderer {
    look: String,
}

impl CharacterRenderer {
    pub fn new() -> Self {
        Self {
            look: String::new(),
        }
    }

    /// Ask the assistant for a character look description.
    pub fn assist_look(&mut self, ai: &AiAssistant, prompt: &str) {
        self.look = ai.assist(prompt);
    }

    pub fn look(&self) -> &str {
        &self.look
    }

    /// Walk the bone tree and produce one render item per bone.
    /// Each child's world matrix is parent * local so the skeleton
    /// stays connected.
    pub fn render(&self, root: &Bone) -> Vec<RenderItem> {
        let mut items = Vec::new();
        self.walk(root, &Matrix4::identity(), &mut items);
        items
    }

    fn walk(&self, bone: &Bone, parent: &Matrix4, items: &mut Vec<RenderItem>) {
        let world = bone.world(parent);
        let t = Transform {
            pos: Vec2::new(world.t.x, world.t.y),
            scale: Vec2::new(1.0, 1.0),
            rot: 0.0,
        };
        items.push(RenderItem::from_transform(bone.name(), &t, true));
        for child in bone.children() {
            self.walk(child, &world, items);
        }
    }
}

impl Default for CharacterRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connects_children() {
        let mut root = Bone::new("root").unwrap();
        root.set_local_translation(0.0, 0.0, 0.0);
        let mut child = Bone::new("spine").unwrap();
        child.set_local_translation(0.0, 1.0, 0.0);
        root.add_child(child);
        let r = CharacterRenderer::new();
        let items = r.render(&root);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].source(), "root");
        assert_eq!(items[1].source(), "spine");
        assert_eq!(items[1].translation(), (0.0, 1.0));
    }

    #[test]
    fn ai_look() {
        let mut r = CharacterRenderer::new();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        r.assist_look(&ai, "describe the character");
        assert!(!r.look().is_empty());
    }

    #[test]
    fn differential() {
        let mut root = Bone::new("root").unwrap();
        let child = Bone::new("arm").unwrap();
        root.add_child(child);
        let r = CharacterRenderer::new();
        let items = r.render(&root);
        assert_eq!(items.len(), 2);
    }
}
