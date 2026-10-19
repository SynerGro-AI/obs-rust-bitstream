//! Safe pure MenuDispatch (menu click → action).
//!
//! A Photoshop menu item click produces an action.
//! Unknown items are rejected. No unwraps.
//!
//! Differential: a different item produces a different action.

use crate::ps_menus::PsMenu;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    NewLayer,
    DuplicateLayer,
    LayerMask,
    GroupLayers,
    MergeDown,
    SelectAll,
    Deselect,
    Inverse,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Other { menu: String, item: String },
}

pub struct MenuDispatch;

impl MenuDispatch {
    /// Dispatch a menu item click. Returns None if unknown.
    pub fn click(menu: PsMenu, item: &str) -> Option<MenuAction> {
        let known = menu.items().iter().any(|(n, _)| *n == item);
        if !known {
            return None;
        }
        Some(match (menu, item) {
            (PsMenu::Layer, "New Layer") => MenuAction::NewLayer,
            (PsMenu::Layer, "Duplicate Layer") => MenuAction::DuplicateLayer,
            (PsMenu::Layer, "Layer Mask") => MenuAction::LayerMask,
            (PsMenu::Layer, "Group Layers") => MenuAction::GroupLayers,
            (PsMenu::Layer, "Merge Down") => MenuAction::MergeDown,
            (PsMenu::Select, "All") => MenuAction::SelectAll,
            (PsMenu::Select, "Deselect") => MenuAction::Deselect,
            (PsMenu::Select, "Inverse") => MenuAction::Inverse,
            (PsMenu::Edit, "Undo") => MenuAction::Undo,
            (PsMenu::Edit, "Redo") => MenuAction::Redo,
            (PsMenu::View, "Zoom In") => MenuAction::ZoomIn,
            (PsMenu::View, "Zoom Out") => MenuAction::ZoomOut,
            _ => MenuAction::Other {
                menu: menu.label().to_string(),
                item: item.to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_dispatches() {
        assert_eq!(
            MenuDispatch::click(PsMenu::Layer, "New Layer"),
            Some(MenuAction::NewLayer)
        );
        assert_eq!(
            MenuDispatch::click(PsMenu::Layer, "Layer Mask"),
            Some(MenuAction::LayerMask)
        );
    }

    #[test]
    fn unknown_rejected() {
        assert!(MenuDispatch::click(PsMenu::Layer, "NotAThing").is_none());
        assert!(MenuDispatch::click(PsMenu::File, "New Layer").is_none());
    }

    #[test]
    fn differential() {
        let a = MenuDispatch::click(PsMenu::Edit, "Undo");
        let b = MenuDispatch::click(PsMenu::Edit, "Redo");
        assert_ne!(a, b);
        assert_eq!(a, Some(MenuAction::Undo));
        assert_eq!(b, Some(MenuAction::Redo));
    }
}
