//! Safe pure RenderList (OBS draw list core).
//!
//! A list of render items built from a scene's items. Hidden items
//! are skipped. No unwraps.
//!
//! Differential: build from scene + count round-trips.

use crate::render_item::RenderItem;
use crate::scene_context::SceneContext;

#[derive(Clone, Debug, Default)]
pub struct RenderList {
    items: Vec<RenderItem>,
}

impl RenderList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a render list from a scene. Hidden items are skipped.
    pub fn from_scene(scene: &SceneContext) -> Self {
        let mut items = Vec::new();
        for item in scene.items() {
            if item.visible() {
                items.push(RenderItem::from_transform(
                    item.source(),
                    item.transform(),
                    true,
                ));
            }
        }
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[RenderItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_context::SceneContext;

    #[test]
    fn from_scene() {
        let mut s = SceneContext::new("Main").unwrap();
        s.add_item("a", "cam").unwrap();
        s.add_item("b", "overlay").unwrap();
        let list = RenderList::from_scene(&s);
        assert_eq!(list.len(), 2);
        assert_eq!(list.items()[0].source(), "cam");
    }

    #[test]
    fn skips_hidden() {
        let mut s = SceneContext::new("Main").unwrap();
        s.add_item("a", "cam").unwrap();
        s.get_mut("a").unwrap().set_visible(false);
        let list = RenderList::from_scene(&s);
        assert!(list.is_empty());
    }

    #[test]
    fn differential() {
        let mut s = SceneContext::new("Main").unwrap();
        s.add_item("a", "cam").unwrap();
        s.get_mut("a").unwrap().set_pos(10.0, 20.0);
        let list = RenderList::from_scene(&s);
        assert_eq!(list.items()[0].translation(), (10.0, 20.0));
    }
}
