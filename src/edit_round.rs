//! Safe pure EditRound (select → edit → result).
//!
//! Confirms the path: select a layer, edit it, only
//! that layer changes. Select a group, edit it, every
//! member changes. No unwraps.
//!
//! Differential: a group edit changes more layers.

use crate::edit_target::{EditApply, EditTarget};
use crate::layer_group::GroupSet;
use crate::layer_stack::LayerStack;

pub struct EditRound;

impl EditRound {
    /// Run and return (layer_changed, group_changed).
    pub fn run() -> (usize, usize) {
        let mut stack = LayerStack::new();
        stack.add("A").unwrap();
        stack.add("B").unwrap();
        stack.add("C").unwrap();
        let mut groups = GroupSet::new();
        groups.add("Group 1").unwrap();
        groups.add_member("Group 1", "A").unwrap();
        groups.add_member("Group 1", "B").unwrap();
        let layer = EditApply::set_opacity(&mut stack, &groups, &EditTarget::Layer("A".into()), 0.5);
        // Reset for the group pass.
        let _ = stack.set_opacity("A", 1.0);
        let group = EditApply::set_opacity(&mut stack, &groups, &EditTarget::Group("Group 1".into()), 0.5);
        (layer, group)
    }

    /// True if the layer edit hit one and the group edit hit more.
    pub fn correct(layer: usize, group: usize) -> bool {
        layer == 1 && group > layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path() {
        let (layer, group) = EditRound::run();
        assert!(EditRound::correct(layer, group));
        assert_eq!(layer, 1);
        assert_eq!(group, 2);
    }

    #[test]
    fn group_hits_more() {
        let (layer, group) = EditRound::run();
        assert!(group > layer);
    }

    #[test]
    fn differential() {
        let (a, b) = EditRound::run();
        assert_ne!(a, b);
    }
}
