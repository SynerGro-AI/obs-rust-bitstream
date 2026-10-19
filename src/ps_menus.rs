//! Safe pure PsMenus (Photoshop menu bar).
//!
//! File, Edit, Image, Layer, Select, Filter, View,
//! Window, Help. Items map to suite modules where
//! present. No unwraps.
//!
//! Differential: a different menu has different items.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PsMenu {
    File,
    Edit,
    Image,
    Layer,
    Select,
    Filter,
    View,
    Window,
    Help,
}

impl PsMenu {
    pub fn label(&self) -> &'static str {
        match self {
            PsMenu::File => "File",
            PsMenu::Edit => "Edit",
            PsMenu::Image => "Image",
            PsMenu::Layer => "Layer",
            PsMenu::Select => "Select",
            PsMenu::Filter => "Filter",
            PsMenu::View => "View",
            PsMenu::Window => "Window",
            PsMenu::Help => "Help",
        }
    }

    pub fn items(&self) -> &'static [(&'static str, Option<&'static str>)] {
        match self {
            PsMenu::File => &[
                ("New", None),
                ("Open", None),
                ("Save", None),
                ("Export", None),
            ],
            PsMenu::Edit => &[
                ("Undo", None),
                ("Redo", None),
                ("Cut", None),
                ("Copy", None),
                ("Paste", None),
                ("Free Transform", Some("transform")),
            ],
            PsMenu::Image => &[
                ("Adjustments", None),
                ("Canvas Size", None),
                ("Image Size", None),
            ],
            PsMenu::Layer => &[
                ("New Layer", Some("layer_stack")),
                ("Duplicate Layer", Some("layer_stack")),
                ("Layer Mask", Some("layer_mask")),
                ("Group Layers", Some("layer_group")),
                ("Merge Down", Some("layer_stack")),
            ],
            PsMenu::Select => &[
                ("All", Some("selection")),
                ("Deselect", Some("selection")),
                ("Inverse", Some("selection")),
                ("Modify", Some("selection")),
            ],
            PsMenu::Filter => &[
                ("Blur", None),
                ("Noise", Some("noise_filter")),
                ("Distort", Some("distort")),
                ("Stylize", None),
            ],
            PsMenu::View => &[
                ("Zoom In", None),
                ("Zoom Out", None),
                ("Fit on Screen", None),
                ("Show Rulers", None),
            ],
            PsMenu::Window => &[
                ("Layers", Some("ps_layers")),
                ("Tools", Some("ps_toolbar")),
                ("History", None),
            ],
            PsMenu::Help => &[("About", None)],
        }
    }
}

pub struct PsMenus;

impl PsMenus {
    pub fn menus() -> &'static [PsMenu] {
        &[
            PsMenu::File,
            PsMenu::Edit,
            PsMenu::Image,
            PsMenu::Layer,
            PsMenu::Select,
            PsMenu::Filter,
            PsMenu::View,
            PsMenu::Window,
            PsMenu::Help,
        ]
    }

    pub fn line() -> String {
        Self::menus()
            .iter()
            .map(|m| m.label())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menus_present() {
        assert_eq!(PsMenus::menus().len(), 9);
        assert!(PsMenus::line().contains("Layer"));
        assert!(PsMenus::line().contains("Filter"));
    }

    #[test]
    fn layer_items_map() {
        let items = PsMenu::Layer.items();
        assert!(items.iter().any(|(n, m)| *n == "Layer Mask" && *m == Some("layer_mask")));
        assert!(items.iter().any(|(n, m)| *n == "Group Layers" && *m == Some("layer_group")));
    }

    #[test]
    fn differential() {
        assert!(PsMenu::Layer.items().len() > PsMenu::Help.items().len());
    }
}
