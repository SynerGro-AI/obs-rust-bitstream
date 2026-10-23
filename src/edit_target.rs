//! Safe pure EditTarget (layer or group).
//!
//! Select a layer and edits hit only that layer.
//! Select a group and edits hit every member.
//! Missing targets are rejected. No unwraps.
//!
//! Differential: a group edit changes more layers
//! than a single-layer edit.

use crate::layer_group::GroupSet;
use crate::layer_stack::LayerStack;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditTarget {
    Layer(String),
    Group(String),
}

pub struct EditApply;

impl EditApply {
    /// Set opacity on the target. A layer sets one.
    /// A group sets every member. Returns the count changed.
    pub fn set_opacity(
        stack: &mut LayerStack,
        groups: &GroupSet,
        target: &EditTarget,
        opacity: f32,
    ) -> usize {
        match target {
            EditTarget::Layer(name) => {
                if stack.set_opacity(name, opacity).is_ok() {
                    1
                } else {
                    0
                }
            }
            EditTarget::Group(name) => {
                let members = match groups.get(name) {
                    Ok(g) => g.members().to_vec(),
                    Err(_) => return 0,
                };
                let mut n = 0;
                for m in members {
                    if stack.set_opacity(&m, opacity).is_ok() {
                        n += 1;
                    }
                }
                n
            }
        }
    }

    /// Names the target will edit.
    pub fn targets(groups: &GroupSet, target: &EditTarget) -> Vec<String> {
        match target {
            EditTarget::Layer(name) => vec![name.clone()],
            EditTarget::Group(name) => groups
                .get(name)
                .map(|g| g.members().to_vec())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_edits_one() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        s.add("B").unwrap();
        let g = GroupSet::new();
        let n = EditApply::set_opacity(&mut s, &g, &EditTarget::Layer("A".into()), 0.5);
        assert_eq!(n, 1);
        assert!(
            (s.layers()
                .iter()
                .find(|l| l.name() == "A")
                .unwrap()
                .opacity()
                - 0.5)
                .abs()
                < 0.01
        );
        assert!(
            (s.layers()
                .iter()
                .find(|l| l.name() == "B")
                .unwrap()
                .opacity()
                - 1.0)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn group_edits_members() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        s.add("B").unwrap();
        s.add("C").unwrap();
        let mut g = GroupSet::new();
        g.add("Group 1").unwrap();
        g.add_member("Group 1", "A").unwrap();
        g.add_member("Group 1", "B").unwrap();
        let n = EditApply::set_opacity(&mut s, &g, &EditTarget::Group("Group 1".into()), 0.3);
        assert_eq!(n, 2);
        assert!(
            (s.layers()
                .iter()
                .find(|l| l.name() == "C")
                .unwrap()
                .opacity()
                - 1.0)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn missing_is_zero() {
        let mut s = LayerStack::new();
        let g = GroupSet::new();
        assert_eq!(
            EditApply::set_opacity(&mut s, &g, &EditTarget::Layer("Nope".into()), 0.5),
            0
        );
    }

    #[test]
    fn differential() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        s.add("B").unwrap();
        let mut g = GroupSet::new();
        g.add("G").unwrap();
        g.add_member("G", "A").unwrap();
        g.add_member("G", "B").unwrap();
        let one = EditApply::targets(&g, &EditTarget::Layer("A".into()));
        let many = EditApply::targets(&g, &EditTarget::Group("G".into()));
        assert!(many.len() > one.len());
    }
}
